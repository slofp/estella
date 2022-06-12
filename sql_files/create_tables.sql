create table guild_config (
    uid bigint unsigned primary key,
    white_list boolean default false,
    leave_ban boolean default false,
    log_channel_id bigint unsigned null,
    auth_role_id bigint unsigned null,
    bot_role_id bigint unsigned null
);

create table main_account (
    uid bigint unsigned primary key,
    name nvarchar(32) not null,
    guild_id bigint unsigned not null,
    version tinyint unsigned not null,
    join_date datetime not null,
    is_sc boolean default false,
    is_leaved boolean default false,
    constraint main_fk_guild_id
        foreign key (guild_id)
        references guild_config (uid)
        on delete cascade on update cascade
);

create table sub_account (
    uid bigint unsigned primary key,
    name nvarchar(32) not null,
    guild_id bigint unsigned not null,
    join_date datetime not null,
    main_uid bigint unsigned not null,
    first_cert bigint unsigned not null,
    second_cert bigint unsigned null,
    constraint sub_fk_main_uid
        foreign key (main_uid)
        references main_account (uid)
        on delete cascade on update cascade,
    constraint sub_fk_first_cert
        foreign key (first_cert)
        references main_account (uid)
        on delete cascade on update cascade,
    constraint sub_fk_second_cert
        foreign key (second_cert)
        references main_account (uid)
        on delete set null on update cascade,
    constraint sub_fk_guild_id
        foreign key (guild_id)
        references guild_config (uid)
        on delete cascade on update cascade
);

create table confirmed_account (
    uid bigint unsigned primary key,
    name nvarchar(32) not null,
    guild_id bigint unsigned not null,
    account_type tinyint unsigned not null,
    main_uid bigint unsigned null,
    first_cert bigint unsigned null,
    second_cert bigint unsigned null,
    constraint confirmed_fk_main_uid
        foreign key (main_uid)
        references main_account (uid)
        on delete cascade on update cascade,
    constraint confirmed_fk_first_cert
        foreign key (first_cert)
        references main_account (uid)
        on delete set null on update cascade,
    constraint confirmed_fk_second_cert
        foreign key (second_cert)
        references main_account (uid)
        on delete set null on update cascade,
    constraint confirmed_fk_guild_id
        foreign key (guild_id)
        references guild_config (uid)
        on delete cascade on update cascade
);

create table pending_account (
    uid bigint unsigned primary key,
    name nvarchar(32) not null,
    guild_id bigint unsigned not null,
    account_type tinyint unsigned not null,
    message_id bigint unsigned not null,
    end_voting datetime null,
    main_uid bigint unsigned null,
    first_cert bigint unsigned null,
    constraint pending_fk_guild_id
        foreign key (guild_id)
        references guild_config (uid)
        on delete cascade on update cascade,
    constraint pending_fk_main_uid
        foreign key (main_uid)
        references main_account (uid)
        on delete cascade on update cascade,
    constraint pending_fk_first_cert
        foreign key (first_cert)
        references main_account (uid)
        on delete set null on update cascade
);

create table user_data (
    uid bigint unsigned primary key,
    glacialeur nvarchar(14) null
);

/*
create table level (
    uid bigint unsigned primary key,
    level bigint unsigned as (floor(((-1940 + sqrt(pow(1940, 2) - 4 * 20 * (-950 - exp))) / (2 * 20)) + 1 )),
    exp double default 0.0
);
*/
