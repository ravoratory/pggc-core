use std::process::Command;

use dotenv::dotenv;
use tonic::{metadata::MetadataValue, transport::Server, Request, Response, Status};

use judgement::judger_server::{Judger, JudgerServer};
use judgement::{JudgeRequest, JudgeResponse};

pub mod judgement {
    tonic::include_proto!("judgement");
}

fn run_judge_script(team: &str, problem: &str) -> (bool, String) {
    // FIXME: ssh にしたいね
    let _ = Command::new("git")
        .args([
            "clone",
            "-q",
            &format!("https://github.com/pggc2-problems/{}.git", problem),
        ])
        .status();

    let judge_result = Command::new("pytest")
        .args(["-q", "--tb=line", "-rN"])
        .arg(format!("test-script/{}.py", problem))
        .output()
        .unwrap();
    // HACK: 整形したかったら pytest-json-report 使うのが良さそう
    let log = String::from_utf8_lossy(&judge_result.stdout).to_string();

    // HACK: clone を何回も実行しないように、開発時は コメントアウト
    let _ = Command::new("rm").args(["-rf", problem]).status();

    return (judge_result.status.success(), log);
}

#[derive(Debug, Default)]
pub struct MyJudger {}

#[tonic::async_trait]
impl Judger for MyJudger {
    async fn judge(
        &self,
        request: Request<JudgeRequest>,
    ) -> Result<Response<JudgeResponse>, Status> {
        let team = &request.get_ref().team;
        // let problem: &u8 = &request.get_ref().problem_id.parse().unwrap();
        let problem = &request.get_ref().problem_name;
        dbg!(team, problem);

        let (judge_status, judge_log) = run_judge_script(team, problem);
        dbg!(&judge_status);
        let response = judgement::JudgeResponse {
            is_correct: judge_status.to_string(),
            log: judge_log,
        };

        Ok(Response::new(response))
    }
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token_string =
        std::env::var("VERIFY_TOKEN").expect("You should set variables 'VERIFY_TOKEN'");
    let token: MetadataValue<_> = format!("Bearer {token_string}", token_string = token_string)
        .parse()
        .unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let server = MyJudger::default();
    dotenv().ok();

    Server::builder()
        .add_service(JudgerServer::with_interceptor(server, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}
