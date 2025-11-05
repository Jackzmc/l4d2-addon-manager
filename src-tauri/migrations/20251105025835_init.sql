-- Add migration script here
create table addons
(
    filename   text    not null,
    created_at integer not null,
    updated_at integer not null,
    file_size  integer not null,

    flags integer not null default 0, -- Bit field

    workshop_id integer,

    primary key (filename)
);

create table addon_tags
(
    filename text not null  references addons (filename) ON UPDATE CASCADE ON DELETE CASCADE,
    tag      text not null,
    primary key (filename, tag)
);

create table workshop_items
(
    publishedfileid integer not null,
    title           text    not null,
    primary key(publishedfileid)
);


