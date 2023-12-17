## Step by Step Guide to Custom Networking


### Step 1. Create a VPC with two public and two private subnets

```
apiVersion: eksctl.io/v1alpha5
kind: ClusterConfig

metadata:
  name: cluster-2
  region: us-west-2

availabilityZones: ["us-west-2a", "us-west-2b"]

vpc:
  cidr: 192.168.0.0/24
  autoAllocateIPv6: true
  hostnameType: resource-name
  # disable public access to endpoint and only allow private access
  clusterEndpoints:
    publicAccess: true
    privateAccess: true
    
managedNodeGroups:
- name: ng1
  availabilityZones: ["us-west-2a"]
- name: ng2
  availabilityZones: ["us-west-2b"]
```

### Step 2. Get the VPC ID

```
vpc_id=$(aws eks describe-cluster --name cluster --query "cluster.resourcesVpcConfig.vpcId" --output text)
```

### Step 2a. Describe the VPC CIDR blocks

```
aws ec2 describe-vpcs --vpc-ids $vpc_id \
    --query 'Vpcs[*].CidrBlockAssociationSet[*].{CIDRBlock: CidrBlock, State: CidrBlockState.State}' --out table
```

### Step 3. Associate a secondary CIDR block with the VPC

```
aws ec2 associate-vpc-cidr-block --vpc-id $vpc_id --cidr-block 192.168.1.0/24
aws ec2 associate-vpc-cidr-block --vpc-id  $vpc_id --cidr-block 100.64.0.0/16
```

### Step 3a. Describe the associated CIDR blocks

```
aws ec2 describe-vpcs --vpc-ids $vpc_id --query 'Vpcs[*].CidrBlockAssociationSet[*].{CIDRBlock: CidrBlock, State: CidrBlockState.State}' --out table
```

### Step 4: Create two private subnets in the primary CIDR block

```
export vpc_id=vpc-0afdadb823bc685ac
export az_1=us-west-2a
export az_2=us-west-2b

new_subnet_id_1=$(aws ec2 create-subnet --vpc-id $vpc_id --availability-zone $az_1 --cidr-block 192.168.1.0/27 \
    --tag-specifications 'ResourceType=subnet,Tags=[{Key=Name,Value=my-eks-custom-networking-vpc-PrivateSubnet01},{Key=kubernetes.io/role/internal-elb,Value=1}]' \
    --query Subnet.SubnetId --output text)
new_subnet_id_2=$(aws ec2 create-subnet --vpc-id $vpc_id --availability-zone $az_2 --cidr-block 192.168.1.32/27 \
    --tag-specifications 'ResourceType=subnet,Tags=[{Key=Name,Value=my-eks-custom-networking-vpc-PrivateSubnet02},{Key=kubernetes.io/role/internal-elb,Value=1}]' \
    --query Subnet.SubnetId --output text)
```

### Step 4a: Ensure the subnets are created.

```
aws ec2 describe-subnets --filters "Name=vpc-id,Values=$vpc_id"     --query 'Subnets[*].{SubnetId: SubnetId,AvailabilityZone: AvailabilityZone,CidrBlock: CidrBlock}'     --output table
---------------------------------------------------------------------
|                          DescribeSubnets                          |
+------------------+-------------------+----------------------------+
| AvailabilityZone |     CidrBlock     |         SubnetId           |
+------------------+-------------------+----------------------------+
|  us-west-2b      |  192.168.0.32/27  |  subnet-08254cc4dd1cbfd0d  |
|  us-west-2a      |  192.168.1.0/27   |  subnet-0c96cc20403123e7f  |
|  us-west-2b      |  192.168.0.96/27  |  subnet-0924ac3127e9cd3ff  |
|  us-west-2b      |  192.168.1.32/27  |  subnet-0f21459f4d74ae438  |
|  us-west-2a      |  192.168.0.0/27   |  subnet-0a50a9b842ed066bc  |
|  us-west-2a      |  192.168.0.64/27  |  subnet-0bf11fb47b15ab227  |
+------------------+-------------------+----------------------------+
```

### Step 5

```
kubectl set env daemonset aws-node -n kube-system AWS_VPC_K8S_CNI_CUSTOM_NETWORK_CFG=true
```

### Step 6

```
cluster_security_group_id=$(aws eks describe-cluster --name $cluster_name --query cluster.resourcesVpcConfig.clusterSecurityGroupId --output text)
```

### Step 7 Create ENIs

```
cat >$az_1.yaml <<EOF
apiVersion: crd.k8s.amazonaws.com/v1alpha1
kind: ENIConfig
metadata:
name: $az_1
spec:
securityGroups:
- $cluster_security_group_id
subnet: $new_subnet_id_1
EOF
```

```
cat >$az_2.yaml <<EOF
apiVersion: crd.k8s.amazonaws.com/v1alpha1
kind: ENIConfig
metadata: 
  name: $az_2
spec: 
  securityGroups: 
    - $cluster_security_group_id
  subnet: $new_subnet_id_2
EOF
```

```
kubectl apply -f $az_1.yaml
kubectl apply -f $az_2.yaml
```

### Step 8 - Recycle the nodes

```
eksctl scale nodegroup --cluster=CLUSTER-NAME --nodes=0 NG-NAME
eksctl scale nodegroup --cluster=CLUSTER-NAME --nodes=2 NG-NAME
```

### Step 9 - Verify Custom Networing

```
kubectl get pods -A -o wide
```