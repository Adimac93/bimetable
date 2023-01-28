create table users (
    id uuid default gen_random_uuid(),
    username text not null,
    primary key (id)
);

create table credentials (
    login text,
    password text not null,
    user_id uuid not null unique,
    primary key (login),
    foreign key (user_id) references users(id)
);
