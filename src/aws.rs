use std::io;

pub fn get_cluster_security_group(cluster_name: &mut String) -> io::Result<String>  {
    let describe_cluster = format!("aws eks describe-cluster --name {} --query cluster.resourcesVpcConfig.clusterSecurityGroupId --output text", cluster_name);

    match crate::utils::execute_capture_output(&*describe_cluster) {
        Ok(output) => return Ok(output.strip_suffix("\n").unwrap().to_string()),
        Err(err) => return Err(err),
    }
}


pub fn get_cluster_vpc(cluster_name: &mut String) -> io::Result<String>  {
    let describe_cluster = format!("aws eks describe-cluster --name {} --query cluster.resourcesVpcConfig.vpcId --output text", cluster_name);

    match crate::utils::execute_capture_output(&*describe_cluster) {
        Ok(output) => return Ok(output.strip_suffix("\n").unwrap().to_string()),
        Err(err) => return Err(err),
    }
}

pub fn associate_cidr_block_with_vpc(vpc_id: &str, cidr_block: &str) -> io::Result<String> {
    let associate_cidr_block = format!("aws ec2 associate-vpc-cidr-block --vpc-id {} --cidr-block {}", vpc_id, cidr_block);
    println!("Associate CIDR Block: {}", associate_cidr_block);

    match crate::utils::execute_capture_output(&*associate_cidr_block) {
        Ok(output) => return Ok(output),
        Err(err) => return Err(err),
    }
}

pub fn create_subnet(vpc_id: &str, az_1: &str, cidr_block: &str) -> io::Result<String> {
    let create_subnet = format!("aws ec2 create-subnet --vpc-id {} --availability-zone {} --cidr-block {} --tag-specifications ResourceType=subnet,Tags=[{{Key=Name,Value=my-eks-custom-networking-vpc-PrivateSubnet01}},{{Key=kubernetes.io/role/internal-elb,Value=1}}] --query Subnet.SubnetId --output text", vpc_id, az_1, cidr_block);
    println!("Create Subnet: {}", create_subnet);

    match crate::utils::execute_capture_output(&*create_subnet) {
        Ok(output) => return Ok(output),
        Err(err) => return Err(err),
    }
}

pub fn describe_vpcs(vpc_id: &str) -> io::Result<String> {
    let describe_vpcs = format!(r#"aws ec2 describe-vpcs --vpc-ids {} --query Vpcs[*].CidrBlockAssociationSet[*] --output table"#, vpc_id);
    println!("Describe VPCs: {}", describe_vpcs);

    match crate::utils::execute_capture_output(&*describe_vpcs) {
        Ok(output) => return Ok(output),
        Err(err) => return Err(err),
    }

}
