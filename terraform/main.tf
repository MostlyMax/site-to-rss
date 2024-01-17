/* networking boilerplate */

resource "aws_vpc" "main" {
    cidr_block           = "10.0.0.0/16"
    enable_dns_hostnames = true

    tags = {
        name = "main"
    }
}

resource "aws_subnet" "subnet_a" {
    vpc_id                  = aws_vpc.main.id
    cidr_block              = cidrsubnet(aws_vpc.main.cidr_block, 8, 1)
    map_public_ip_on_launch = true
    availability_zone       = "us-east-1a"
}

resource "aws_subnet" "subnet_b" {
    vpc_id                  = aws_vpc.main.id
    cidr_block              = cidrsubnet(aws_vpc.main.cidr_block, 8, 2)
    map_public_ip_on_launch = true
    availability_zone       = "us-east-1b"
}

resource "aws_internet_gateway" "internet_gateway" {
    vpc_id = aws_vpc.main.id
    tags = {
        Name = "internet_gateway"

    }
}

resource "aws_route_table" "route_table" {
    vpc_id = aws_vpc.main.id

    route {
        cidr_block = "0.0.0.0/0"
        gateway_id = aws_internet_gateway.internet_gateway.id
    }
}

resource "aws_route_table_association" "subnet_a_route" {
    subnet_id      = aws_subnet.subnet_a.id
    route_table_id = aws_route_table.route_table.id
}

resource "aws_route_table_association" "subnet_b_route" {
    subnet_id      = aws_subnet.subnet_b.id
    route_table_id = aws_route_table.route_table.id
}

// update for tighter rules
resource "aws_security_group" "security_group" {
    name   = "ecs-security-group"
    vpc_id = aws_vpc.main.id

    ingress {
        from_port   = 0
        to_port     = 0
        protocol    = -1
        self        = "false"
        cidr_blocks = ["0.0.0.0/0"]
        description = "any"
    }

    egress {
        from_port   = 0
        to_port     = 0
        protocol    = "-1"
        cidr_blocks = ["0.0.0.0/0"]
    }
}
// ---------------------------------------------------------------
/* compute */

resource "aws_launch_template" "ecs_lt" {
    name_prefix   = "ecs-template"
    image_id      = "ami-0c0b74d29acd0cd97"
    instance_type = "t3.medium"

    key_name               = "ed25519"
    vpc_security_group_ids = [aws_security_group.security_group.id]

    # iam_instance_profile {
    #     name = "ecsInstanceRole"
    # }

    block_device_mappings {
        device_name = "/dev/xvda"

        ebs {
            volume_size = 10
            volume_type = "gp2"
        }
    }

    tag_specifications {
        resource_type = "instance"
        tags = {
            Name = "ecs-instance"
        }
    }

    user_data = filebase64("${path.module}/ecs.sh")
}

resource "aws_lb" "ecs_alb" {
    name               = "ecs-alb"
    internal           = false
    load_balancer_type = "application"
    security_groups    = [aws_security_group.security_group.id]
    subnets            = [aws_subnet.subnet_a.id, aws_subnet.subnet_b.id]

    tags = {
        Name = "ecs-alb"
    }
}

resource "aws_lb_listener" "ecs_alb_listener" {
    load_balancer_arn = aws_lb.ecs_alb.arn
    port              = 80
    protocol          = "HTTP"

    default_action {
        type             = "forward"
        target_group_arn = aws_lb_target_group.ecs_tg.arn
    }
}

resource "aws_lb_target_group" "ecs_tg" {
    name        = "ecs-target-group"
    port        = 80
    protocol    = "HTTP"
    target_type = "ip"
    vpc_id      = aws_vpc.main.id

    health_check {
        path = "/health"
    }
}

resource "aws_ecs_cluster" "ecs_cluster" {
    name = "site-to-rss-ecs-cluster"
}

resource "aws_ecs_task_definition" "ecs_task_definition" {
    family                   = "site-to-rss-docker"
    network_mode             = "awsvpc"
    execution_role_arn       = "arn:aws:iam::917404528856:role/ecsTaskExecutionRole"
    requires_compatibilities = ["FARGATE"]

    cpu    = 1024
    memory = 2048

    runtime_platform {
        operating_system_family = "LINUX"
        cpu_architecture        = "X86_64"
    }


    container_definitions = jsonencode([{
        name      = "site-to-rss-docker"
        image     = "917404528856.dkr.ecr.us-east-1.amazonaws.com/site-to-rss-ecr:latest"
        # cpu       = 1024
        essential = true

        portMappings = [
            {
                containerPort = 9000
                hostPort      = 9000
                protocol      = "tcp"
            }
        ]
    }])
}

resource "aws_ecs_service" "ecs_service" {
    name            = "site-to-rss-service"
    cluster         = aws_ecs_cluster.ecs_cluster.id
    task_definition = aws_ecs_task_definition.ecs_task_definition.arn
    launch_type     = "FARGATE"
    desired_count   = 1

    load_balancer {
        target_group_arn = aws_lb_target_group.ecs_tg.arn
        container_name   = aws_ecs_task_definition.ecs_task_definition.family
        container_port   = 9000
    }

    network_configuration {
        subnets          = [aws_subnet.subnet_a.id, aws_subnet.subnet_b.id]
        assign_public_ip = true
        security_groups  = [aws_security_group.security_group.id]
    }
}
