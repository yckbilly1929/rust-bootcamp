pub mod auth;

use crate::{
    pb::{
        RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest,
        WelcomeResponse,
    },
    CrmService,
};
use chrono::{Duration, Utc};
use crm_metadata::pb::{metadata_client::MetadataClient, Content, MaterializeRequest};
use crm_send::pb::SendRequest;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Channel, Response, Status};
use tracing::warn;
use user_stat::pb::QueryRequest;

const CHANNEL_SIZE: usize = 1024;

impl CrmService {
    pub async fn welcome(&self, req: WelcomeRequest) -> Result<Response<WelcomeResponse>, Status> {
        let request_id = req.id;
        let d1 = Utc::now() - Duration::days(req.interval as _);
        let d2 = d1 + Duration::days(1);
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = get_contents_by_id(self.metadata.clone(), &req.content_ids).await?;

        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Welcome".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                }
            }
        });
        let reqs = ReceiverStream::new(rx);

        // NOTE: this is an alternative solution
        // let sender = self.config.server.sender_email.clone();
        // let reqs = res.filter_map(move |v| {
        //     let sender: String = sender.clone();
        //     let contents = contents.clone();
        //     async move {
        //         let v = v.ok()?;
        //         Some(gen_send_req("Welcome".to_string(), sender, v, &contents))
        //     }
        // });

        self.notification.clone().send(reqs).await?;

        Ok(Response::new(WelcomeResponse { id: request_id }))
    }

    pub async fn recall(&self, req: RecallRequest) -> Result<Response<RecallResponse>, Status> {
        let request_id = req.id.clone();
        let d1 = Utc::now() - Duration::minutes(req.last_visit_interval as _); // NOTE: minutes instead of days
        let d2 = Utc::now();
        let query = QueryRequest::new_with_dt("last_visited_at", d1, d2);
        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let contents = get_contents_by_id(self.metadata.clone(), &req.content_ids).await?;

        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = res_user_stats.next().await {
                let contents = contents.clone();
                let sender = sender.clone();
                let tx = tx.clone();

                let req = SendRequest::new("Recall".to_string(), sender, &[user.email], &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("failed to send message: {:?}", e);
                }
            }
        });
        let reqs = ReceiverStream::new(rx);

        self.notification.clone().send(reqs).await?;

        Ok(Response::new(RecallResponse { id: request_id }))
    }

    pub async fn remind(&self, req: RemindRequest) -> Result<Response<RemindResponse>, Status> {
        let request_id = req.id.clone();
        let d1 = Utc::now() - Duration::minutes(req.last_visit_interval as _); // NOTE: minutes instead of days
        let d2 = Utc::now();
        let mut query = QueryRequest::new_with_dt("last_visited_at", d1, d2);
        if !req.content_ids.is_empty() {
            query.add_content_ids_constraint("started_but_not_finished", req.content_ids);
        }

        let mut res_user_stats = self.user_stats.clone().query(query).await?.into_inner();

        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        let metadata = self.metadata.clone();
        let sender = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            // TODO: is it good to batch call instead?
            while let Some(Ok(user)) = res_user_stats.next().await {
                // TODO: viewed_but_not_started as well?
                if let Some(needed_contents) = user.contents.get("started_but_not_finished") {
                    match get_contents_by_id(metadata.clone(), &needed_contents.ids).await {
                        Ok(contents) => {
                            let req = SendRequest::new(
                                "Remind".to_string(),
                                sender.clone(),
                                &[user.email],
                                &contents,
                            );
                            if let Err(e) = tx.send(req).await {
                                warn!("failed to send message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get_contents_by_id: {:?}", e);
                        }
                    }
                }
            }
        });
        let reqs = ReceiverStream::new(rx);

        self.notification.clone().send(reqs).await?;

        Ok(Response::new(RemindResponse { id: request_id }))
    }
}

// TODO: move internal service elsewhere?
async fn get_contents_by_id(
    metadata: MetadataClient<Channel>,
    ids: &[u32],
) -> Result<Arc<Vec<Content>>, Status> {
    let contents = metadata
        .clone()
        .materialize(MaterializeRequest::new_with_ids(ids))
        .await?
        .into_inner();

    let contents: Vec<Content> = contents
        .filter_map(|v| async move { v.ok() })
        .collect()
        .await;
    let contents = Arc::new(contents);
    Ok(contents)
}
