use bollard::{
    Docker,
    exec::CreateExecOptions,
    container::{LogsOptions, LogOutput},
    errors::Error };
use futures::Stream;
use std::time::SystemTime;
use serde::Serialize;

pub struct Server {
    pub name: String,
    pub path: String,
    pub rcon: String,
    pub id: String,
}

impl Server {
    pub async fn send_command(&self, cmd: Vec<&str>) -> Result<String, String> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

        let mut full_cmd = vec!["rcon-cli"];
            full_cmd.extend(cmd);
        let exec = docker
        .create_exec(
            &self.id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(full_cmd),
                ..Default::default()
            },
        )
        .await.unwrap()
        .id;

        if let Ok(_) = docker.start_exec(&exec, None).await {
            return Ok(String::from("Successfully sent cmd to container"));
        } else {
            return Err(String::from("Failed to send cmd to container"));
        }
    }

    pub async fn start(&self) -> Result<String, String> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();
        
        if let Ok(_) = docker.start_container::<String>(&self.id, None).await {
            return Ok(String::from("Successfully started the container"));
        } else {
            return Err(String::from("Failed to start the container"));
        }
    }

    pub async fn stop(&self) -> Result<String, String> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().unwrap();

        if let Ok(_) = docker.stop_container(&self.id, None).await {
            return Ok(String::from("Successfully stopped the container"));
        } else {
            return Err(String::from("Failed to stop the container"));
        }
    }

    pub fn status(&self) {
        unimplemented!();
    }

   pub fn output(&self) -> impl Stream<Item = Result<LogOutput, Error>> {
       #[cfg(unix)]
       let docker = Docker::connect_with_socket_defaults().unwrap();

       // fix to give only new logs
       let options = Some(LogsOptions::<String>{
            stdout: true,
            //since: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as i64,
            follow: true,
            ..Default::default()
        });

        docker.logs(&self.id, options)
    }
}
