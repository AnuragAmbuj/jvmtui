use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use jvm_tui::{
    app::App,
    cli::Cli,
    jvm::{
        connector::JvmConnector, discovery::discover_local_jvms,
        jdk_tools::connector::JdkToolsConnector,
    },
    metrics::{collector::MetricsCollector, store::MetricsStore},
    tui::screens::{jvm_picker::JvmPickerScreen, monitoring::MonitoringScreen},
    tui::terminal,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let jvms = discover_local_jvms().await?;

    if jvms.is_empty() {
        println!("No JVM processes found.");
        println!("Make sure you have running Java applications.");
        return Ok(());
    }

    let mut terminal = terminal::setup_terminal()?;
    let mut picker = JvmPickerScreen::new(jvms.clone());

    let selected_jvm = loop {
        terminal.draw(|frame| {
            picker.render(frame);
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
                        if let Some(jvm) = picker.selected_jvm() {
                            break jvm.clone();
                        }
                    }
                    (KeyCode::Char('r'), _) => {
                        let jvms = discover_local_jvms().await?;
                        picker = JvmPickerScreen::new(jvms);
                    }
                    _ => {}
                }
            }
        }
    };

    let mut connector = JdkToolsConnector::new();
    connector.connect(selected_jvm.pid).await?;

    let jvm_info = connector.get_jvm_info().await?;

    let interval = cli.interval.unwrap_or(Duration::from_secs(1));
    let store = Arc::new(RwLock::new(MetricsStore::new(300)));
    let mut app = App::new(store.clone());
    app.set_jvm_info(jvm_info);

    let connector_arc: Arc<RwLock<dyn JvmConnector>> = Arc::new(RwLock::new(connector));
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
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        break;
                    }
                    (KeyCode::Char('1'), _) => app.select_tab(0),
                    (KeyCode::Char('2'), _) => app.select_tab(1),
                    (KeyCode::Char('3'), _) => app.select_tab(2),
                    (KeyCode::Char('4'), _) => app.select_tab(3),
                    (KeyCode::Char('5'), _) => app.select_tab(4),
                    (KeyCode::Char('l'), _) | (KeyCode::Tab, _) => app.next_tab(),
                    (KeyCode::Char('h'), _) | (KeyCode::BackTab, _) => app.previous_tab(),
                    (KeyCode::Char('g'), _) => {
                        let conn = connector_arc.read().await;
                        let _ = conn.trigger_gc().await;
                    }
                    (KeyCode::Char('r'), _) => {
                        let mut store_mut = store.write().await;
                        *store_mut = MetricsStore::new(100);
                    }
                    _ => {}
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
