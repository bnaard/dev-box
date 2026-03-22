---
name: terraform-basics
description: Infrastructure-as-code with Terraform/OpenTofu. Resources, providers, state, modules, and plan/apply workflow. Use when writing Terraform configs, managing cloud infrastructure, or reviewing IaC code.
allowed-tools: Bash(terraform:*) Bash(tofu:*) Read Write
---

# Terraform Basics

## When to Use

- Writing or editing Terraform/OpenTofu configuration files (`.tf`)
- Planning, applying, or destroying cloud infrastructure
- Managing state files and backend configuration
- Creating or consuming reusable modules
- Reviewing IaC code for best practices
- Debugging provider or resource issues

## Instructions

### Resources and Providers

Every Terraform project starts with a provider configuration. Pin provider versions in a `required_providers` block:

```hcl
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "eu-central-1"
}
```

Resources are the core building blocks. Use descriptive names and always set critical arguments explicitly rather than relying on defaults.

### Variables and Outputs

Use typed variables with descriptions and validation where appropriate:

```hcl
variable "instance_type" {
  type        = string
  default     = "t3.micro"
  description = "EC2 instance size"
  validation {
    condition     = can(regex("^t3\\.", var.instance_type))
    error_message = "Only t3 instance types are allowed."
  }
}

output "instance_ip" {
  value       = aws_instance.web.public_ip
  description = "Public IP of the web server"
}
```

Variable types: `string`, `number`, `bool`, `list(type)`, `map(type)`, `object({...})`. Use `terraform.tfvars` or `-var-file` for environment-specific values. Never hardcode secrets -- use environment variables or a secrets manager data source.

### Data Sources

Use data sources to reference existing infrastructure without managing it:

```hcl
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"]
  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }
}
```

### State Management

State tracks the mapping between config and real infrastructure. Configure a remote backend for team use:

```hcl
terraform {
  backend "s3" {
    bucket         = "myproject-tfstate"
    key            = "prod/terraform.tfstate"
    region         = "eu-central-1"
    dynamodb_table = "terraform-locks"
    encrypt        = true
  }
}
```

Never edit state files manually. Use `terraform state list`, `terraform state show`, and `terraform state mv` for inspection and refactoring. Use `terraform import` to bring existing resources under management.

### Modules

Modules group related resources for reuse. Keep modules focused on a single concern:

```hcl
module "vpc" {
  source  = "./modules/vpc"
  cidr    = "10.0.0.0/16"
  azs     = ["eu-central-1a", "eu-central-1b"]
}
```

Public modules from the registry: `source = "terraform-aws-modules/vpc/aws"` with a `version` pin. Always pin module versions in production.

### Plan / Apply / Destroy Workflow

Always preview before applying:

```bash
terraform init          # download providers and modules
terraform fmt -check    # check formatting
terraform validate      # syntax and internal consistency check
terraform plan -out=tfplan   # preview changes, save plan
terraform apply tfplan       # apply the saved plan exactly
```

Use `terraform destroy` with extreme caution -- review the plan output first. For targeted operations, use `-target=resource.name` sparingly and only for debugging.

### Best Practices

- One state per environment (dev/staging/prod) via workspaces or directory structure
- Use `lifecycle { prevent_destroy = true }` on critical resources (databases, S3 buckets)
- Tag all resources with project, environment, and owner
- Run `terraform fmt` and `terraform validate` before committing
- Use `moved` blocks for refactoring instead of destroy/recreate

## Examples

### Provision an EC2 instance with security group

```hcl
resource "aws_security_group" "web" {
  name_prefix = "web-"
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_instance" "web" {
  ami                    = data.aws_ami.ubuntu.id
  instance_type          = var.instance_type
  vpc_security_group_ids = [aws_security_group.web.id]
  tags = { Name = "web-server", Environment = "prod" }
}
```

### Migrate state backend from local to S3

```bash
# 1. Add backend config to terraform block
# 2. Run init with migration flag
terraform init -migrate-state
# 3. Verify state is accessible
terraform state list
# 4. Delete local terraform.tfstate after confirming
```

### Import an existing resource into state

```bash
# Find the resource ID from the cloud console or CLI
terraform import aws_s3_bucket.logs my-existing-log-bucket
# Write the matching resource block in .tf
terraform plan  # should show no changes if config matches
```
