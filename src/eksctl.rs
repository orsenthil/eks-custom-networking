use crate::utils;

pub fn verify_ng_scaled_down(cluster_name: &mut String, ng1_name: &String) {
    let describe_ng1 = format!("eksctl get nodegroup --cluster {} --region us-west-2 --name {} -o yaml", cluster_name, ng1_name);
    let mut desired_capacity = false;

    while !desired_capacity {
        let output = utils::execute_capture_output(&*describe_ng1).unwrap();
        let output: Vec<&str> = output.split("\n").collect();
        for line in output {
            if line.contains("DesiredCapacity: 0") {
                desired_capacity = true;
            }
        }
    }
}
