import * as cdk from "@aws-cdk/core";
import * as cw from "@aws-cdk/aws-cloudwatch";
import { CfnOutput } from "@aws-cdk/core";

export interface DashboardStackProps extends cdk.StackProps {
    name: string
}

export class DashboardStack extends cdk.Stack {
    
    dashboard: cw.Dashboard;
    
    constructor(scope: cdk.Construct, id: string, props: DashboardStackProps) {
        super(scope, id, props);
        
        
        this.dashboard = new cw.Dashboard(this, id + "Dashboard", {
            dashboardName: props.name
        });
    }
}
