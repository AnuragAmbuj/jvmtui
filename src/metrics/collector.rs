use crate::error::Result;
use crate::jvm::connector::JvmConnector;
use crate::metrics::store::MetricsStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

pub struct MetricsCollector {
    connector: Arc<RwLock<dyn JvmConnector>>,
    store: Arc<RwLock<MetricsStore>>,
    interval: Duration,
    tick_count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl MetricsCollector {
    pub fn new(
        connector: Arc<RwLock<dyn JvmConnector>>,
        store: Arc<RwLock<MetricsStore>>,
        interval: Duration,
    ) -> Self {
        Self {
            connector,
            store,
            interval,
            tick_count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let mut ticker = interval(self.interval);

        loop {
            ticker.tick().await;

            let tick = self
                .tick_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let connector = self.connector.read().await;
            if !connector.is_connected().await {
                break;
            }

            if let Ok(heap_info) = connector.get_heap_info().await {
                let mut store = self.store.write().await;
                store.record_heap(heap_info);
            }

            if let Ok(gc_stats) = connector.get_gc_stats().await {
                let mut store = self.store.write().await;
                store.record_gc(gc_stats);
            }

            if let Ok(thread_info) = connector.get_thread_info().await {
                let mut store = self.store.write().await;
                store.record_threads(thread_info);
            }

            if tick % 10 == 0 {
                if let Ok(class_histogram) = connector.get_class_histogram().await {
                    let mut store = self.store.write().await;
                    store.record_class_histogram(class_histogram);
                }
            }
        }

        Ok(())
    }

    pub async fn collect_once(&self) -> Result<()> {
        let connector = self.connector.read().await;
        if !connector.is_connected().await {
            return Err(crate::error::AppError::Connection(
                "Not connected".to_string(),
            ));
        }

        if let Ok(heap_info) = connector.get_heap_info().await {
            let mut store = self.store.write().await;
            store.record_heap(heap_info);
        }

        if let Ok(gc_stats) = connector.get_gc_stats().await {
            let mut store = self.store.write().await;
            store.record_gc(gc_stats);
        }

        if let Ok(thread_info) = connector.get_thread_info().await {
            let mut store = self.store.write().await;
            store.record_threads(thread_info);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jvm::jdk_tools::connector::JdkToolsConnector;

    #[tokio::test]
    async fn test_metrics_collector() {
        let jvms = crate::jvm::discovery::discover_local_jvms().await.unwrap();
        if jvms.is_empty() {
            println!("No JVMs found, skipping test");
            return;
        }

        let mut connector = JdkToolsConnector::new();
        connector.connect(jvms[0].pid).await.unwrap();

        let connector: Arc<RwLock<dyn JvmConnector>> = Arc::new(RwLock::new(connector));
        let store = Arc::new(RwLock::new(MetricsStore::new(10)));

        let collector =
            MetricsCollector::new(connector.clone(), store.clone(), Duration::from_secs(1));

        collector.collect_once().await.unwrap();

        let store_read = store.read().await;
        assert!(store_read.heap_history.len() > 0);
        assert!(store_read.gc_history.len() > 0);

        println!("Collected {} heap samples", store_read.heap_history.len());
        println!("Collected {} GC samples", store_read.gc_history.len());
    }

    #[tokio::test]
    async fn test_continuous_collection() {
        let jvms = crate::jvm::discovery::discover_local_jvms().await.unwrap();
        if jvms.is_empty() {
            println!("No JVMs found, skipping test");
            return;
        }

        let mut connector = JdkToolsConnector::new();
        connector.connect(jvms[0].pid).await.unwrap();

        let connector: Arc<RwLock<dyn JvmConnector>> = Arc::new(RwLock::new(connector));
        let store = Arc::new(RwLock::new(MetricsStore::new(10)));

        let collector =
            MetricsCollector::new(connector.clone(), store.clone(), Duration::from_millis(100));

        let collector_handle = tokio::spawn(async move { collector.run().await });

        tokio::time::sleep(Duration::from_millis(350)).await;

        {
            let mut conn = connector.write().await;
            conn.disconnect().await.unwrap();
        }

        let _ = tokio::time::timeout(Duration::from_secs(1), collector_handle).await;

        let store_read = store.read().await;
        println!(
            "Collected {} heap samples in 350ms",
            store_read.heap_history.len()
        );
        assert!(store_read.heap_history.len() >= 1);
    }
}
