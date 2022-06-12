insert into <table_name>
	(row_name...)
values
	(insert_row_val...)
on duplicate key update
	update_set_row = val,
	...
;
