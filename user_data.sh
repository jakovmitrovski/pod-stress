#!/bin/bash

echo "Debug: Environment variables"
env

echo "Debug: AWS credentials"
aws sts get-caller-identity

# Install and configure NTP
yum update -y
yum install -y chrony
systemctl enable chronyd
systemctl start chronyd

# Configure chrony for high precision
cat > /etc/chrony.conf << EOF
makestep 1 3
rtcsync
local stratum 10
allow
EOF

# Restart chrony with new config
systemctl restart chronyd

# Wait for chrony to sync
sleep 5

# Install Docker
yum install -y docker
systemctl start docker
systemctl enable docker

sudo yum install -y at
sudo systemctl enable atd
sudo systemctl start atd


# Add ec2-user to docker group
usermod -a -G docker ec2-user

# Debug: Check Docker status
echo "Debug: Docker status"
systemctl status docker

# Debug: Check Docker installation
echo "Debug: Docker version"
docker --version

echo "Debug: Testing ECR access"
aws ecr get-login-password --region eu-central-1 | docker login --username AWS --password-stdin 971228983574.dkr.ecr.eu-central-1.amazonaws.com


echo "Debug: Pulling replica image"
docker pull 971228983574.dkr.ecr.eu-central-1.amazonaws.com/stress:latest


cat <<EOF > /home/ec2-user/run_stress.sh
#!/bin/bash

docker run \\
  -e RPC_URL_WS="wss://rpc.dev.pod.network" \\
  -e PRIVATE_KEY="e0233a902ff84ebda2fdba5a8083a45afee1a5604861f92279e9a5f5ee8519e3" \\
  -e CONTRACT_ADDRESS="0x4CF3F1637bfEf1534e56352B6ebAae243aF464c3" \\
  -e TZ=UTC \\
  --cap-add SYS_TIME \\
  --cap-add SYS_NICE \\
  971228983574.dkr.ecr.eu-central-1.amazonaws.com/stress:latest
EOF

chmod +x /home/ec2-user/run_stress.sh

for i in {1..20}; do
  echo "Running iteration $i"
  ./home/ec2-user/run_stress.sh
done
