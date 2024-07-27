use std::sync::Arc;
use tokio::time::{Duration, interval};
use futures::{Stream, StreamExt};
use async_stream::stream;
use neo::builder::TransactionError;
use neo::prelude::{JsonRpcClient, NeoBlock<Transaction>};
use crate::prelude::Middleware;

pub struct JsonRpc2_Rx {
    provider: Arc<dyn JsonRpcClient<Error=TransactionError>>,
}

impl JsonRpc2_Rx {
    pub fn new(provider: Arc<dyn JsonRpcClient<Error=TransactionError>>) -> Self {
        Self { provider }
    }

    pub fn block_index_stream(&self, polling_interval: Duration) -> impl Stream<Item = Result<u32, Box<dyn std::error::Error>>> {
        stream! {
            let mut interval = interval(polling_interval);
            loop {
                interval.tick().await;
                match self.provider.as_ref().get_block_count().await {
                    Ok(count) => yield Ok(count - 1),
                    Err(e) => yield Err(Box::new(e) as Box<dyn std::error::Error>),
                }
            }
        }
    }

    pub fn block_stream(&self, full_transaction_objects: bool, polling_interval: Duration) -> impl Stream<Item = Result<NeoBlock<Transaction>, Box<dyn std::error::Error>>> {
        self.block_index_stream(polling_interval)
            .then(move |result| {
                async move {
                    match result {
                        Ok(index) => self.provider.as_ref().get_block_by_index(index, full_transaction_objects).await
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
                        Err(e) => Err(e),
                    }
                }
            })
    }

    pub fn replay_blocks_stream(&self, start_block: u32, end_block: u32, full_transaction_objects: bool, ascending: bool) -> impl Stream<Item = Result<NeoBlock<Transaction>, Box<dyn std::error::Error>>> {
        let range: Vec<u32> = if ascending {
            (start_block..=end_block).collect()
        } else {
            (start_block..=end_block).rev().collect()
        };

        stream! {
            for block in range {
                match self.provider.as_ref().get_block_by_index(block, full_transaction_objects).await {
                    Ok(block) => yield Ok(block),
                    Err(e) => yield Err(Box::new(e) as Box<dyn std::error::Error>),
                }
            }
        }
    }

    pub fn catch_up_to_latest_block_stream<S>(&self, start_block: u32, full_transaction_objects: bool, on_caught_up_stream: S) -> impl Stream<Item = Result<NeoBlock<Transaction>, Box<dyn std::error::Error>>>
    where
        S: Stream<Item = Result<NeoBlock<Transaction>, Box<dyn std::error::Error>>> + Send + 'static,
    {
        stream! {
            let latest_block = match self.provider.as_ref().get_block_count().await {
                Ok(count) => count - 1,
                Err(e) => {
                    yield Err(Box::new(e) as Box<dyn std::error::Error>);
                    return;
                }
            };

            if start_block >= latest_block {
                for await item in on_caught_up_stream {
                    yield item;
                }
            } else {
                let replay_stream = self.replay_blocks_stream(start_block, latest_block, full_transaction_objects, true);
                for await item in replay_stream {
                    yield item;
                }
                let catch_up_stream = self.catch_up_to_latest_block_stream(latest_block + 1, full_transaction_objects, on_caught_up_stream);
                for await item in catch_up_stream {
                    yield item;
                }
            }
        }
    }

    pub fn catch_up_to_latest_and_subscribe_to_new_blocks_stream(&self, start_block: u32, full_transaction_objects: bool, polling_interval: Duration) -> impl Stream<Item = Result<NeoBlock<Transaction>, Box<dyn std::error::Error>>> {
        self.catch_up_to_latest_block_stream(
            start_block,
            full_transaction_objects,
            self.block_stream(full_transaction_objects, polling_interval),
        )
    }

    pub fn latest_block_index_stream(&self) -> impl Stream<Item = Result<u32, Box<dyn std::error::Error>>> {
        stream! {
            match self.provider.as_ref().get_block_count().await {
                Ok(count) => yield Ok(count - 1),
                Err(e) => yield Err(Box::new(e) as Box<dyn std::error::Error>),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::neo_providers::rx::json_rpc_rx::JsonRpc2_Rx;

    #[tokio::test]
    async fn test_replay_blocks_observable() {
        let mock_url_session = Arc::new(MockURLSession::new());
        let neo_rust = Arc::new(MockNeoRust { mock_url_session: mock_url_session.clone() });
        let json_rpc = JsonRpc2_Rx::new(neo_rust);

        let neo_get_blocks = vec![
            MockBlocks::create_block(0),
            MockBlocks::create_block(1),
            MockBlocks::create_block(2),
        ];

        for block in &neo_get_blocks {
            mock_url_session.add_response(serde_json::to_string(block).unwrap());
        }

        let mut results = Vec::new();
        let mut stream = json_rpc.replay_blocks_stream(0, 2, false, true);

        while let Some(result) = stream.next().await {
            results.push(result.unwrap());
        }

        assert_eq!(results.len(), neo_get_blocks.len());
        for (result, expected) in results.iter().zip(neo_get_blocks.iter()) {
            assert_eq!(result.block, expected.block);
        }
    }

    #[tokio::test]
    async fn test_replay_blocks_descending_observable() {
        let mock_url_session = Arc::new(MockURLSession::new());
        let neo_rust = Arc::new(MockNeoRust { mock_url_session: mock_url_session.clone() });
        let json_rpc = JsonRpc2_Rx::new(neo_rust);

        let neo_get_blocks = vec![
            MockBlocks::create_block(2),
            MockBlocks::create_block(1),
            MockBlocks::create_block(0),
        ];

        for block in &neo_get_blocks {
            mock_url_session.add_response(serde_json::to_string(block).unwrap());
        }

        let mut results = Vec::new();
        let mut stream = json_rpc.replay_blocks_stream(0, 2, false, false);

        while let Some(result) = stream.next().await {
            results.push(result.unwrap());
        }

        assert_eq!(results.len(), neo_get_blocks.len());
        for (result, expected) in results.iter().zip(neo_get_blocks.iter()) {
            assert_eq!(result.block, expected.block);
        }
    }

    #[tokio::test]
    async fn test_catch_up_to_latest_and_subscribe_to_new_block_observable() {
        let mock_url_session = Arc::new(MockURLSession::new());
        let neo_rust = Arc::new(MockNeoRust { mock_url_session: mock_url_session.clone() });
        let json_rpc = JsonRpc2_Rx::new(neo_rust);

        let neo_get_blocks = vec![
            MockBlocks::create_block(0),
            MockBlocks::create_block(1),
            MockBlocks::create_block(2),
            MockBlocks::create_block(3),
            MockBlocks::create_block(4),
            MockBlocks::create_block(5),
            MockBlocks::create_block(6),
        ];

        let mut block_count = NeoBlockCount::new(4);
        mock_url_session.add_response(serde_json::to_string(&block_count).unwrap());
        for block in &neo_get_blocks {
            mock_url_session.add_response(serde_json::to_string(block).unwrap());
        }

        let mut results = Vec::new();
        let mut stream = json_rpc.catch_up_to_latest_and_subscribe_to_new_blocks_stream(0, false, Duration::from_secs(1));

        tokio::spawn(async move {
            for _ in 4..7 {
                sleep(Duration::from_secs(2)).await;
                block_count = NeoBlockCount::new(block_count.block_count + 1);
                mock_url_session.add_response(serde_json::to_string(&block_count).unwrap());
            }
        });

        let timeout = tokio::time::sleep(Duration::from_secs(10));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
            Some(result) = stream.next() => {
                results.push(result.unwrap());
            }
            _ = &mut timeout => break,
        }
        }

        assert_eq!(results.len(), neo_get_blocks.len());
        for (result, expected) in results.iter().zip(neo_get_blocks.iter()) {
            assert_eq!(result.block, expected.block);
        }
    }

    #[tokio::test]
    async fn test_subscribe_to_new_block_observable() {
        let mock_url_session = Arc::new(MockURLSession::new());
        let neo_rust = Arc::new(MockNeoRust { mock_url_session: mock_url_session.clone() });
        let json_rpc = JsonRpc2_Rx::new(neo_rust);

        let neo_get_blocks = vec![
            MockBlocks::create_block(0),
            MockBlocks::create_block(1),
            MockBlocks::create_block(2),
            MockBlocks::create_block(3),
        ];

        let mut block_count = NeoBlockCount::new(0);
        mock_url_session.add_response(serde_json::to_string(&block_count).unwrap());
        for block in &neo_get_blocks {
            mock_url_session.add_response(serde_json::to_string(block).unwrap());
        }

        let mut results = Vec::new();
        let mut stream = json_rpc.subscribe_to_new_blocks_stream(false, Duration::from_secs(1));

        tokio::spawn(async move {
            for _ in 0..4 {
                sleep(Duration::from_secs(2)).await;
                block_count = NeoBlockCount::new(block_count.block_count + 1);
                mock_url_session.add_response(serde_json::to_string(&block_count).unwrap());
            }
        });

        let timeout = tokio::time::sleep(Duration::from_secs(10));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
            Some(result) = stream.next() => {
                results.push(result.unwrap());
            }
            _ = &mut timeout => break,
        }
        }

        assert_eq!(results.len(), neo_get_blocks.len());
        for (result, expected) in results.iter().zip(neo_get_blocks.iter()) {
            assert_eq!(result.block, expected.block);
        }
    }
}