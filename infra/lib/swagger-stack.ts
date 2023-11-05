import { Construct } from 'constructs';
import * as cdk from "aws-cdk-lib";
import * as cw from "aws-cdk-lib/aws-cloudwatch";
import * as cognito from "aws-cdk-lib/aws-cognito";
import { CfnOutput } from "aws-cdk-lib";

export interface SwaggerStackProps extends cdk.StackProps {
    user_pool_id: string;
    resource_server: cognito.IUserPoolResourceServer;
    scope: cognito.ResourceServerScope;
}

export class SwaggerStack extends cdk.Stack {
    user_pool: cognito.IUserPool;
    
    constructor(scope: Construct, id: string, props: SwaggerStackProps) {
        super(scope, id, props);
        
        this.user_pool = cognito.UserPool.fromUserPoolId(this, "UserPool", props.user_pool_id);

        this.user_pool.addClient("swagger", {
            oAuth: {
                flows: {
                    authorizationCodeGrant: true,
                },
                scopes: [
                    cognito.OAuthScope.OPENID,
                    cognito.OAuthScope.EMAIL,
                    cognito.OAuthScope.resourceServer(props.resource_server, props.scope),
                ],
                callbackUrls: [
                    "https://app.swaggerhub.com/oauth2_redirect"
                ]
            },
            userPoolClientName: "wagon-swagger",
            generateSecret: true,
            supportedIdentityProviders: [
                cognito.UserPoolClientIdentityProvider.COGNITO,
            ]
        });
    }
}
