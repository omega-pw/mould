--用户
create table "user"
(
    id uuid not null primary key,
    org_id uuid, --组织id
    user_source smallint not null, --用户来源
    name varchar(128) not null, --名称
    avatar_url text, --头像
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--系统用户
create table "system_user"
(
    id uuid not null primary key,
    email varchar(256) not null, --邮箱
    user_random_value varchar(128) not null, --随机数
    hashed_auth_key varchar(128) not null, --授权秘钥摘要
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--外部用户
create table external_user
(
    id uuid not null primary key,
    provider_type smallint not null, --提供者类型
    provider varchar(128) not null, --提供者
    openid text not null, --开放id
    detail text, --详细信息
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--组织
create table organization
(
    id uuid not null primary key,
    name varchar(128) not null, --名称
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--环境规格
create table environment_schema
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    name varchar(128) not null, --名称
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--环境规格资源
create table environment_schema_resource
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    environment_schema_id uuid not null, --环境规格id
    name varchar(128) not null, --资源名称
    extension_id varchar(512) not null, --扩展id
    extension_name varchar(512) not null, --扩展名称
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--环境
create table environment
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    environment_schema_id uuid not null, --环境规格id
    name varchar(128) not null, --环境名称
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--环境资源
create table environment_resource
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    environment_id uuid not null, --环境id
    schema_resource_id uuid not null, --环境规格资源id
    name varchar(128) not null, --资源名称
    extension_id varchar(512) not null, --扩展id
    extension_name varchar(512) not null, --扩展名称
    extension_configuration text not null, --扩展配置
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--任务
create table job
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    environment_schema_id uuid not null, --环境规格id
    name varchar(128) not null, --任务名称
    remark text, --备注
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--任务步骤
create table job_step
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    job_id uuid not null, --任务id
    name varchar(128) not null, --步骤名称
    step_type smallint not null, --步骤类型
    schema_resource_id uuid not null, --环境规格资源id
    operation_id varchar(512) not null, --操作id
    operation_name varchar(512) not null, --操作名称
    operation_parameter text not null, --操作参数
    attachments text, --附件
    remark text, --备注
    seq int4 not null, --执行顺序
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--任务记录
create table job_record
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    job_id uuid not null, --任务id
    environment_id uuid not null, --环境id
    status smallint not null, --执行状态
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--任务步骤记录
create table job_step_record
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    job_id uuid not null, --任务id
    environment_id uuid not null, --环境id
    record_id uuid not null, --记录id
    job_step_id uuid not null, --任务步骤id
    step_name varchar(128) not null, --步骤名称
    step_type smallint not null, --步骤类型
    step_remark text, --步骤备注
    extension_id varchar(512) not null, --扩展id
    operation_id varchar(512) not null, --操作id
    operation_name varchar(512) not null, --操作名称
    operation_parameter text not null, --操作参数
    attachments text, --附件
    job_step_seq int4 not null, --任务步骤顺序
    status smallint not null, --执行状态
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);

--任务步骤资源记录
create table job_step_resource_record
(
    id uuid not null primary key,
    org_id uuid not null, --组织id
    job_id uuid not null, --任务id
    environment_id uuid not null, --环境id
    record_id uuid not null, --记录id
    job_step_record_id uuid not null, --任务步骤记录id
    environment_resource_id uuid not null, --环境资源id
    resource_name varchar(128) not null, --资源名称
    extension_configuration text not null, --扩展配置
    output_file varchar(256), --日志文件
    output_content text, --日志内容
    status smallint not null, --执行状态
    created_time timestamptz not null,
    last_modified_time timestamptz not null
);
