-- Add migration script here
create table addons
(
    filename text,
    created_at integer not null,
    updated_at integer not null,
    file_size  integer not null,

    -- extracted addon info
    title text not null,
    author text,
    version text not null,
    tagline text,
    flags integer not null default 0, -- Bit field

    workshop_id integer, -- extracted from addoninfo.txt or filename

    primary key (title, version)
);

create table addon_tags
(
    title text not null,
    version text not null,
    tag      text not null,
    primary key (title, version, tag)
    foreign key (title, version) references addons (title, version)  ON UPDATE CASCADE ON DELETE CASCADE
);

create table workshop_items
(
    publishedfileid integer not null,
    title           text    not null,
    primary key(publishedfileid)
);


