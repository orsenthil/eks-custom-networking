use crate::utils;

pub fn ensure_pods_running(expected: i32) {
    let mut pods_running = false;
    while !pods_running {
        let pods = utils::execute_capture_output("kubectl get pods -A -o wide").unwrap();
        let pods: Vec<&str> = pods.split("\n").collect();
        let mut pods_running_count = 0;
        for pod in pods {
            if pod.contains("Running") {
                pods_running_count += 1;
            }
        }
        if pods_running_count >= expected{
            pods_running = true;
        }
    }
}
