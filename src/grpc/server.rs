use crate::grpc::task_scheduler::{
    ExecuteTaskRequest, ExecuteTaskResponse, QueryTaskRequest, QueryTaskResponse,
    RegisterTaskRequest, RegisterTaskResponse, TaskStatus,
    scheduler_server::{Scheduler, SchedulerServer},
};
use tonic::{Request, Response, Status};

pub struct SchedulerService;

#[tonic::async_trait]
impl Scheduler for SchedulerService {
    async fn register_task(
        &self,
        request: Request<RegisterTaskRequest>,
    ) -> Result<Response<RegisterTaskResponse>, Status> {
        let req = request.into_inner();
        println!("Register task: {:?}", req);

        Ok(Response::new(RegisterTaskResponse {
            success: true,
            message: "Registered".into(),
        }))
    }

    async fn execute_task(
        &self,
        request: Request<ExecuteTaskRequest>,
    ) -> Result<Response<ExecuteTaskResponse>, Status> {
        let req = request.into_inner();
        println!("Execute task: {:?}", req);

        Ok(Response::new(ExecuteTaskResponse {
            success: true,
            message: "Executed".into(),
        }))
    }

    async fn query_task(
        &self,
        request: Request<QueryTaskRequest>,
    ) -> Result<Response<QueryTaskResponse>, Status> {
        let req = request.into_inner();
        println!("Query task: {:?}", req);

        Ok(Response::new(QueryTaskResponse {
            status: TaskStatus::TaskRunning as i32,
            last_run_time: "".into(),
            next_run_time: "".into(),
            message: "OK".into(),
        }))
    }
    
}

pub fn create_server() -> SchedulerServer<SchedulerService> {
    SchedulerServer::new(SchedulerService)
}
