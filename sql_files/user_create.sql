-- normal user create
create user '@user'@'@host' identified by '@pass';
grant all on db.* to '@user'@'@host';

-- placeholder user create ?
set @user = 'username', ...;
prepare userc from concat('create user "', @user, '"@"', @host, '" identified by "', @pass, '"');
execute userc;
deallocate prepare userc;

