//! Golden CLI contracts for DevOps fixture parsers (`tests/fixtures/devops/`).

mod common;

use ags_sidecar::contract_parsers::{
    git_status_dirty, parse_cron_line, parse_docker_image_line, parse_systemd_timer_line,
};
use ags_sidecar::services::devops::{parse_docker_ps_line, parse_kubectl_pod_list, CronJob, DockerContainer, DockerImage, KubectlPodSummary, SystemdTimer};
use common::load_fixture;

fn assert_docker_container_contract(c: &DockerContainer) {
    assert!(!c.id.is_empty());
    assert!(!c.name.is_empty());
    assert!(!c.image.is_empty());
    assert!(!c.status.is_empty());
}

fn assert_docker_image_contract(img: &DockerImage) {
    assert!(!img.id.is_empty());
    assert!(!img.repository.is_empty());
    assert!(!img.tag.is_empty());
    assert!(!img.size.is_empty());
}

fn assert_systemd_timer_contract(t: &SystemdTimer) {
    assert!(!t.name.is_empty());
    assert!(!t.next_run.is_empty());
    assert!(!t.last_run.is_empty());
}

fn assert_cron_job_contract(job: &CronJob) {
    assert_eq!(job.schedule.matches(' ').count(), 4);
    assert!(!job.command.is_empty());
    assert!(!job.user.is_empty());
}

fn assert_kubectl_pod_contract(pod: &KubectlPodSummary) {
    assert!(!pod.name.is_empty());
    assert!(!pod.namespace.is_empty());
    assert!(!pod.phase.is_empty());
}

#[test]
fn contract_docker_ps_fixture() {
    let text = load_fixture("devops/docker_ps.txt");
    let containers: Vec<_> = text.lines().filter_map(parse_docker_ps_line).collect();
    assert_eq!(containers.len(), 2);
    for c in &containers {
        assert_docker_container_contract(c);
    }
    assert_eq!(containers[0].ports, "0.0.0.0:8080->80/tcp");
    assert_eq!(containers[1].status, "Exited (0) 1 day ago");
}

#[test]
fn contract_docker_images_fixture() {
    let text = load_fixture("devops/docker_images.txt");
    let images: Vec<_> = text.lines().filter_map(parse_docker_image_line).collect();
    assert_eq!(images.len(), 2);
    for img in &images {
        assert_docker_image_contract(img);
    }
    assert_eq!(images[0].repository, "nginx");
    assert!(parse_docker_image_line("truncated|line").is_none());
}

#[test]
fn contract_systemd_timers_fixture() {
    let text = load_fixture("devops/systemctl_timers.txt");
    let timers: Vec<_> = text.lines().filter_map(parse_systemd_timer_line).collect();
    assert_eq!(timers.len(), 2);
    for t in &timers {
        assert_systemd_timer_contract(t);
    }
    assert!(timers[0].active);
    assert!(!timers[1].active);
    assert!(parse_systemd_timer_line("too few").is_none());
}

#[test]
fn contract_crontab_fixture() {
    let text = load_fixture("devops/crontab_sample.txt");
    let jobs: Vec<_> = text
        .lines()
        .filter_map(|line| parse_cron_line(line, "root"))
        .collect();
    assert_eq!(jobs.len(), 1);
    assert_cron_job_contract(&jobs[0]);
    assert_eq!(jobs[0].command, "/usr/bin/logrotate");
}

#[test]
fn contract_kubectl_pod_list_fixtures() {
    let empty = load_fixture("devops/kubectl_pods_empty.json");
    assert_eq!(parse_kubectl_pod_list(&empty), Some(vec![]));

    let populated = load_fixture("devops/kubectl_pods.json");
    let pods = parse_kubectl_pod_list(&populated).expect("kubectl json");
    assert_eq!(pods.len(), 2);
    for pod in &pods {
        assert_kubectl_pod_contract(pod);
    }
    assert_eq!(pods[0].name, "nginx-abc");
    assert_eq!(pods[1].phase, "Pending");
}

#[test]
fn contract_git_porcelain_dirty_fixture() {
    let clean = load_fixture("devops/git_porcelain_clean.txt");
    let dirty = load_fixture("devops/git_porcelain_dirty.txt");
    assert!(!git_status_dirty(&clean));
    assert!(git_status_dirty(&dirty));
}
