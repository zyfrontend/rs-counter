create table counters (
  id integer primary key not null,
  user_id integer not null,
  name text not null,
  value integer not null,
  step integer not null,
  input_step integer not null,
  sequence integer not null,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

create unique index counters_user_id_index on counters (user_id asc, sequence desc);

create table counter_records (
  id integer primary key not null,
  counter_id integer not null,
  step integer not null,
  'begin' integer not null,
  'end' integer not null,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

create unique index counter_records_counter_id_index on counter_records (counter_id);