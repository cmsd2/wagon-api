import { Construct } from 'constructs';
import * as cdk from "aws-cdk-lib";
import * as cw from "aws-cdk-lib/aws-cloudwatch";
import { CfnOutput } from "aws-cdk-lib";

export interface DashboardStackProps extends cdk.StackProps {
    name: string
}

export class DashboardStack extends cdk.Stack {
    
    dashboard: cw.Dashboard;
    
    constructor(scope: Construct, id: string, props: DashboardStackProps) {
        super(scope, id, props);
        
        
        this.dashboard = new cw.Dashboard(this, id + "Dashboard", {
            dashboardName: props.name
        });
    }
}
