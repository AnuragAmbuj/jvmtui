use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use jvm_tui::{
    app::{App, AppMode, ExportFormat, Tab},
    cli::Cli,
    config::{Config, ConnectionProfile},
    export,
    jvm::{
        connector::JvmConnector,
        discovery::{discover_local_jvms, DiscoveredJvm},
        jdk_tools::connector::JdkToolsConnector,
        jolokia::connector::JolokiaConnector,
    },
    metrics::{collector::MetricsCollector, store::MetricsStore},
    theme::Theme,
    tui::screens::{jvm_picker::JvmPickerScreen, monitoring::MonitoringScreen},
    tui::terminal,
    tui::views::threads::ThreadsView,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

enum SelectedConnection {
    LocalJvm(DiscoveredJvm),
    Jolokia {
        url: String,
        username: Option<String>,
        password: Option<String>,
    },
    SshJdk {
        host: String,
        user: String,
        port: u16,
        key: Option<String>,
        password: Option<String>,
        pid: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let config = if let Some(ref config_path) = cli.config {
        Config::load_from_file(config_path)?
    } else {
        Config::load()?
    };

    let jvms = discover_local_jvms().await?;

    if jvms.is_empty() && config.connections.is_empty() {
        println!("No JVM processes or saved connections found.");
        println!("Make sure you have running Java applications, or");
        println!("add saved connections to your config file.");
        return Ok(());
    }

    let mut terminal = terminal::setup_terminal()?;
    let mut picker = JvmPickerScreen::new(jvms.clone(), config.connections.clone());

    let selected_connection = loop {
        terminal.draw(|frame| {
            picker.render(frame, &Theme);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        terminal::restore_terminal(&mut terminal)?;
                        return Ok(());
                    }
                    (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                        picker.next();
                    }
                    (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                        picker.previous();
                    }
                    (KeyCode::Enter, _) => {
                        // Handle saved connection selection
                        if let Some(conn) = picker.selected_connection() {
                            match conn {
                                ConnectionProfile::Local { pid: Some(pid), .. } => {
                                    // Find the JVM with this PID
                                    if let Some(jvm) = jvms.iter().find(|j| j.pid == *pid) {
                                        break SelectedConnection::LocalJvm(jvm.clone());
                                    } else {
                                        // PID not found, show error and continue
                                        terminal::restore_terminal(&mut terminal)?;
                                        eprintln!("Error: Saved connection references PID {} which is not running", pid);
                                        return Ok(());
                                    }
                                }
                                ConnectionProfile::Local { pid: None, .. } => {
                                    // Local connection without PID - shouldn't happen in valid config
                                    terminal::restore_terminal(&mut terminal)?;
                                    eprintln!("Error: Local connection must specify a PID");
                                    return Ok(());
                                }
                                ConnectionProfile::Jolokia {
                                    url,
                                    username,
                                    password,
                                    ..
                                } => {
                                    break SelectedConnection::Jolokia {
                                        url: url.clone(),
                                        username: username.clone(),
                                        password: password.clone(),
                                    };
                                }
                                ConnectionProfile::SshJdk {
                                    ssh_host,
                                    ssh_user,
                                    ssh_port,
                                    ssh_key,
                                    ssh_password,
                                    pid,
                                    ..
                                } => {
                                    break SelectedConnection::SshJdk {
                                        host: ssh_host.clone(),
                                        user: ssh_user.clone(),
                                        port: *ssh_port,
                                        key: ssh_key.clone(),
                                        password: ssh_password.clone(),
                                        pid: *pid,
                                    };
                                }
                                ConnectionProfile::SshJolokia { .. } => {
                                    terminal::restore_terminal(&mut terminal)?;
                                    println!("SSH+Jolokia tunnel connections coming soon");
                                    println!("For now, use:");
                                    println!("  - Direct Jolokia HTTP");
                                    println!("  - SSH+JDK (jcmd/jstat over SSH)");
                                    println!("  - Local JVMs");
                                    return Ok(());
                                }
                            }
                        }
                        // Handle discovered JVM selection
                        else if let Some(jvm) = picker.selected_jvm() {
                            break SelectedConnection::LocalJvm(jvm.clone());
                        }
                    }
                    (KeyCode::Char('r'), _) => {
                        let jvms = discover_local_jvms().await?;
                        picker = JvmPickerScreen::new(jvms, config.connections.clone());
                    }
                    _ => {}
                }
            }
        }
    };

    let jvm_info;
    let connector_arc: Arc<RwLock<dyn JvmConnector>> = match selected_connection {
        SelectedConnection::LocalJvm(jvm) => {
            let mut connector = JdkToolsConnector::new();
            connector.connect(jvm.pid).await?;
            jvm_info = connector.get_jvm_info().await?;
            Arc::new(RwLock::new(connector))
        }
        SelectedConnection::Jolokia {
            url,
            username,
            password,
        } => {
            let mut connector = JolokiaConnector::new(url, username, password);
            connector.connect(0).await?;
            jvm_info = connector.get_jvm_info().await?;
            Arc::new(RwLock::new(connector))
        }
        SelectedConnection::SshJdk { .. } => {
            terminal::restore_terminal(&mut terminal)?;
            eprintln!("SSH+JDK connections are not yet implemented.");
            eprintln!("This feature requires an SSH library which is coming soon.");
            eprintln!("\nFor now, please use:");
            eprintln!("  - Local JVMs (automatic discovery)");
            eprintln!("  - Direct Jolokia HTTP connections");
            return Ok(());
        }
    };

    let interval = cli.interval.unwrap_or(config.preferences.default_interval);
    let history_size = config.preferences.max_history_samples;
    let store = Arc::new(RwLock::new(MetricsStore::new(history_size)));
    let mut app = App::new(store.clone());
    app.set_jvm_info(jvm_info);
    let collector = MetricsCollector::new(connector_arc.clone(), store.clone(), interval);

    let collector_handle = tokio::spawn(async move {
        let _ = collector.run().await;
    });

    loop {
        let store_snapshot = {
            let store = store.read().await;
            store.clone()
        };

        terminal.draw(|frame| {
            MonitoringScreen::render(frame, &app, &store_snapshot);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let CrosstermEvent::Key(key) = event::read()? {
                match app.mode {
                    AppMode::Help => match key.code {
                        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                            app.toggle_help();
                        }
                        _ => {}
                    },
                    AppMode::Error(_) => match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('r') => {
                            app.show_loading("Reconnecting to JVM...".to_string());
                            let mut conn = connector_arc.write().await;
                            match conn.reconnect().await {
                                Ok(_) => {
                                    app.clear_loading();
                                }
                                Err(e) => {
                                    app.show_error(format!("Failed to reconnect: {}", e));
                                }
                            }
                        }
                        _ => {}
                    },
                    AppMode::Loading(_) => {}
                    AppMode::Search => match key.code {
                        KeyCode::Esc => {
                            app.cancel_search();
                        }
                        KeyCode::Enter => {
                            if !app.search_results.is_empty() {
                                app.mode = AppMode::Normal;
                            }
                        }
                        KeyCode::Char('n') if key.modifiers.is_empty() => {
                            app.next_search_result();
                        }
                        KeyCode::Char('N') | KeyCode::Char('n')
                            if key.modifiers.contains(KeyModifiers::SHIFT) =>
                        {
                            app.prev_search_result();
                        }
                        KeyCode::Backspace => {
                            app.pop_search_char();
                            if app.current_tab == Tab::Threads {
                                let store_read = store.read().await;
                                let results =
                                    ThreadsView::search_threads(&store_read, &app.search_query);
                                app.update_search_results(results);
                            }
                        }
                        KeyCode::Char(c) => {
                            app.push_search_char(c);
                            if app.current_tab == Tab::Threads {
                                let store_read = store.read().await;
                                let results =
                                    ThreadsView::search_threads(&store_read, &app.search_query);
                                if !results.is_empty() {
                                    app.scroll_offset = results[0];
                                }
                                app.update_search_results(results);
                            }
                        }
                        _ => {}
                    },
                    AppMode::ConfirmGc => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let conn = connector_arc.read().await;
                            let _ = conn.trigger_gc().await;
                            app.cancel_confirmation();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.cancel_confirmation();
                        }
                        _ => {}
                    },
                    AppMode::SelectExportFormat => match key.code {
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.next_export_format();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.previous_export_format();
                        }
                        KeyCode::Enter => {
                            app.show_export_confirmation();
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.cancel_confirmation();
                        }
                        _ => {}
                    },
                    AppMode::ConfirmExport => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            app.show_loading("Exporting data...".to_string());
                            let store_read = store.read().await;
                            let export_dir = config.preferences.export_directory.as_deref();
                            let result = match app.current_tab {
                                Tab::Threads => export::export_thread_dump(
                                    &store_read.thread_snapshot,
                                    export_dir,
                                ),
                                _ => match app.selected_export_format {
                                    ExportFormat::Json => {
                                        export::export_metrics_json(&store_read, export_dir)
                                    }
                                    ExportFormat::Prometheus => {
                                        export::export_metrics_prometheus(&store_read, export_dir)
                                    }
                                    ExportFormat::Csv => {
                                        export::export_metrics_csv(&store_read, export_dir)
                                    }
                                },
                            };

                            match result {
                                Ok(path) => {
                                    app.show_export_success(path.to_string_lossy().to_string());
                                }
                                Err(e) => {
                                    app.show_error(format!("Export failed: {}", e));
                                }
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.cancel_confirmation();
                        }
                        _ => {}
                    },
                    AppMode::ExportSuccess(_) => match key.code {
                        KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                            app.cancel_confirmation();
                        }
                        _ => {}
                    },
                    AppMode::Normal => match (key.code, key.modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            break;
                        }
                        (KeyCode::Char('?'), _) => {
                            app.toggle_help();
                        }
                        (KeyCode::Char('1'), _) => app.select_tab(0),
                        (KeyCode::Char('2'), _) => app.select_tab(1),
                        (KeyCode::Char('3'), _) => app.select_tab(2),
                        (KeyCode::Char('4'), _) => app.select_tab(3),
                        (KeyCode::Char('5'), _) => app.select_tab(4),
                        (KeyCode::Char('l'), _) | (KeyCode::Tab, _) | (KeyCode::Right, _) => {
                            app.next_tab()
                        }
                        (KeyCode::Char('h'), _) | (KeyCode::BackTab, _) | (KeyCode::Left, _) => {
                            app.previous_tab()
                        }
                        (KeyCode::Char('j'), _) | (KeyCode::Down, _) => {
                            app.scroll_down();
                        }
                        (KeyCode::Char('k'), _) | (KeyCode::Up, _) => {
                            app.scroll_up();
                        }
                        (KeyCode::Char('g'), _) => {
                            app.show_gc_confirmation();
                        }
                        (KeyCode::Char('e'), _) => {
                            if app.current_tab == Tab::Threads {
                                app.show_export_confirmation();
                            } else {
                                app.show_export_format_selector();
                            }
                        }
                        (KeyCode::Char('/'), _) => {
                            if app.current_tab == Tab::Threads {
                                app.start_search();
                            }
                        }
                        (KeyCode::Char('r'), _) => {
                            let mut store_mut = store.write().await;
                            *store_mut = MetricsStore::new(100);
                            app.reset_scroll();
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    {
        let mut conn = connector_arc.write().await;
        conn.disconnect().await?;
    }

    let _ = tokio::time::timeout(Duration::from_secs(1), collector_handle).await;

    terminal::restore_terminal(&mut terminal)?;
    Ok(())
}
