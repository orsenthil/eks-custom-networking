mod aws;
mod utils;
mod k8s;
mod eksctl;

use colored::*;

const ADDITIONAL_VPC_CIDR: &'static str = "100.64.0.0/16";
const ADDITIONAL_VPC_CIDR_SUBNET1: &'static str = "100.64.1.0/17";
const ADDITIONAL_VPC_CIDR_SUBNET2: &'static str = "100.64.128.0/17";

fn main() {
    println!("{}", "Rust program to create an EKS Cluster with Custom Networking Support".yellow());
    println!("--------------------------------------------------------------------");
    println!("{}", "Checking for dependencies...".yellow());
    // Check for aws cli
    println!("{}", "Checking for aws cli...".yellow());
    match utils::execute_capture_output("aws --version") {
        Ok(_) => (),
        Err(_err) => println!("aws cli not found. This program requires aws command line to be present."),
    }

    // Check for eksctl
    println!("{}", "Checking for eksctl...".yellow());
    match utils::execute_capture_output("eksctl version") {
        Ok(_) => (),
        Err(_err) => println!("eksctl not found. This program requires eksctl command line to be present."),
    }

    // Check for kubectl
    println!("{}", "Checking for kubectl...".yellow());
    match utils::execute_capture_output("kubectl version") {
        Ok(_) => (),
        Err(_err) => println!("kubectl not found. This program requires kubectl command line to be present."),
    }

    println!("{}", "Creating a default cluster...".green());
    println!("{}", "EKS Version 1.28".green());
    println!("{}", "VPC CIDR Block:  cidr: 192.168.0.0/24".green());
    println!("{}", "Managed Node Group Ubuntu 20.04".green());
    println!("{}", "Region: us-west-2, Availability Zones: us-west-2a (2 nodes) , us-west-2b (2 nodes)".green());

    // Step 1 - Create a default cluster
    let template = utils::read_file_to_string("templates/cluster.yaml").unwrap();
    let user = std::env::var("USER").unwrap();
    let date = chrono::Local::now().format("%Y-%m-%d-%H").to_string();

    let mut cluster_name = format!("{}-{}", user, date);
    let ng1_name = format!("{}-ng1", cluster_name);
    let ng2_name = format!("{}-ng2", cluster_name);
    let template = template.replace("DEFAULT-CLUSTER-NAME", &*cluster_name);
    utils::write_to_file("cluster-files/cluster.yaml", &*template).unwrap();

    match utils::execute_command("eksctl create cluster -f cluster-files/cluster.yaml") {
        Ok(_) => (),
        Err(err) => println!("Create Cluster Failed with an Error: {:?}", err),
    }

    match utils::execute_capture_output("kubectl get nodes") {
        Ok(output) => println!("{}", output),
        Err(err) => println!("Error: {:?}", err),
    }

    // kubectl set env daemonset aws-node -n kube-system AWS_VPC_K8S_CNI_CUSTOM_NETWORK_CFG=true
    println!("{}", "Setting AWS_VPC_K8S_CNI_CUSTOM_NETWORK_CFG=true".green());
    match utils::execute_command("kubectl set env daemonset aws-node -n kube-system AWS_VPC_K8S_CNI_CUSTOM_NETWORK_CFG=true") {
        Ok(_) => (),
        Err(err) => println!("Failed setting AWS_VPC_K8S_CNI_CUSTOM_NETWORK_CFG=true. Command failed with Error: {:?}", err),
    }

    let vpc_id = aws::get_cluster_vpc(&mut cluster_name);
    let vpc_id_unwrapped = vpc_id.unwrap();

    let security_group = aws::get_cluster_security_group(&mut cluster_name);
    let security_group_unwrapped = security_group.unwrap();

    let _ = aws::describe_vpcs(vpc_id_unwrapped.as_str());
    let _ = aws::associate_cidr_block_with_vpc(vpc_id_unwrapped.as_str(), ADDITIONAL_VPC_CIDR);

    let describe = aws::describe_vpcs(vpc_id_unwrapped.as_str());
    println!("VPC CIDR Blocks\n\n{}", describe.unwrap());

    let subnet1 = aws::create_subnet(vpc_id_unwrapped.as_str(), "us-west-2a", ADDITIONAL_VPC_CIDR_SUBNET1);
    let subnet2 = aws::create_subnet(vpc_id_unwrapped.as_str(), "us-west-2b", ADDITIONAL_VPC_CIDR_SUBNET2);

    let subnet1_template = utils::read_file_to_string("templates/us-west-2a.yaml").unwrap();
    let subnet2_template = utils::read_file_to_string("templates/us-west-2b.yaml").unwrap();

    let subnet1_template = subnet1_template.replace("CLUSTER-SECURITY-GROUP", &*security_group_unwrapped);
    let subnet1_template = subnet1_template.replace("NEW-SUBNET-ID1", &*subnet1.unwrap());
    utils::write_to_file("cluster-files/us-west-2a.yaml", &*subnet1_template).unwrap();

    let subnet2_template = subnet2_template.replace("CLUSTER-SECURITY-GROUP", &*security_group_unwrapped);
    let subnet2_template = subnet2_template.replace("NEW-SUBNET-ID2", &*subnet2.unwrap());
    utils::write_to_file("cluster-files/us-west-2b.yaml", &*subnet2_template).unwrap();

    // Create ENIConfig for us-west-2a
    match utils::execute_command("kubectl apply -f cluster-files/us-west-2a.yaml") {
        Ok(_) => (),
        Err(err) => println!("Failed creating ENIConfig for us-west-2a. Command failed with Error: {:?}", err),
    }

    // Create ENIConfig for us-west-2b
    match utils::execute_command("kubectl apply -f cluster-files/us-west-2b.yaml") {
        Ok(_) => (),
        Err(err) => println!("Failed creating ENIConfig for us-west-2b. Command failed with Error: {:?}", err),
    }

    match utils::execute_command("kubectl get ENIConfigs") {
        Ok(_) => (),
        Err(err) => println!("Failed getting ENIConfigs. Command failed with Error: {:?}", err),
    }

    // kubectl set env daemonset aws-node -n kube-system ENI_CONFIG_LABEL_DEF=topology.kubernetes.io/zone
    println!("{}", "Setting ENI_CONFIG_LABEL_DEF=topology.kubernetes.io/zone".green());

    match utils::execute_command("kubectl set env daemonset aws-node -n kube-system ENI_CONFIG_LABEL_DEF=topology.kubernetes.io/zone") {
        Ok(_) => (),
        Err(err) => println!("Failed setting ENI_CONFIG_LABEL_DEF=topology.kubernetes.io/zone. Command failed with Error: {:?}", err),
    }

    // Recycling the nodes
    let scale_down_ng1= format!("eksctl scale nodegroup --cluster={} --nodes=0 {}", cluster_name, ng1_name);
    match utils::execute_command(&*scale_down_ng1) {
        Ok(_) => (),
        Err(err) => println!("Failed scaling down node group {}. Command failed with Error: {:?}", ng1_name, err),
    }

    eksctl::verify_ng_scaled_down(&mut cluster_name, &ng1_name);

    let scale_up_ng1= format!("eksctl scale nodegroup --cluster={} --nodes=2 {}", cluster_name, ng1_name);
    match utils::execute_command(&*scale_up_ng1) {
        Ok(_) => (),
        Err(err) => println!("Failed scaling up node group {}. Command failed with Error: {:?}", ng1_name, err),
    }


    let scale_down_ng2= format!("eksctl scale nodegroup --cluster={} --nodes=0 {}", cluster_name, ng2_name);
    match utils::execute_command(&*scale_down_ng2) {
        Ok(_) => (),
        Err(err) => println!("Failed scaling down node group {}. Command failed with Error: {:?}", ng2_name, err),
    }

    eksctl::verify_ng_scaled_down(&mut cluster_name, &ng2_name);

    let scale_up_ng2= format!("eksctl scale nodegroup --cluster={} --nodes=2 {}", cluster_name, ng2_name);

    match utils::execute_command(&*scale_up_ng2) {
        Ok(_) => (),
        Err(err) => println!("Failed scaling up node group {}. Command failed with Error: {:?}", ng2_name, err),
    }

    println!("{}", "Verify Custom Networking. Waiting for all pods to be in Running state...".green());
    k8s::ensure_pods_running(10);

    println!("{}", "Deploying sample workloads".green());
    match utils::execute_command("kubectl apply -f workloads/nginx.yaml") {
        Ok(_) => (),
        Err(err) => println!("Failed deploying nginx. Command failed with Error: {:?}", err),
    }

    match utils::execute_command("kubectl apply -f workloads/curl.yaml") {
        Ok(_) => (),
        Err(err) => println!("Failed deploying curl. Command failed with Error: {:?}", err),
    }
    k8s::ensure_pods_running(12);

    match utils::execute_command("kubectl get pods -A -o wide") {
        Ok(_) => (),
        Err(err) => println!("Failed getting pods. Command failed with Error: {:?}", err),
    }

    println!("{}", "Custom Networking is enabled".green());
}