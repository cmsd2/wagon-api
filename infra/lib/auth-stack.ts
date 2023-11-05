import { Construct } from 'constructs';
import * as cdk from "aws-cdk-lib";
import * as cw from "aws-cdk-lib/aws-cloudwatch";
import * as cognito from "aws-cdk-lib/aws-cognito"
import { CfnOutput } from "aws-cdk-lib";

export interface AuthStackProps extends cdk.StackProps {
    user_pool_id: string
}

export class AuthStack extends cdk.Stack {
    
    user_pool: cognito.IUserPool;
    read_scope: cognito.ResourceServerScope;
    full_access_scope: cognito.ResourceServerScope;
    resource_server: cognito.IUserPoolResourceServer;
    
    constructor(scope: Construct, id: string, props: AuthStackProps) {
        super(scope, id, props);
        
        this.user_pool = cognito.UserPool.fromUserPoolId(this, "om", props.user_pool_id);
        
        const read_scope = 'read';
        const full_access_scope = 'write';

        this.read_scope = new cognito.ResourceServerScope({ scopeName: read_scope, scopeDescription: 'Read-only access' });
        this.full_access_scope = new cognito.ResourceServerScope({ scopeName: full_access_scope, scopeDescription: 'Full access' });

        this.resource_server = this.user_pool.addResourceServer("wagon", {
            identifier: "wagon-api",
            scopes: [
                this.read_scope,
                this.full_access_scope,
            ]
        });
    }
}
