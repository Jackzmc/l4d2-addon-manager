-- Add migration script here
create table addons
(
    filename text,
    created_at integer not null,
    updated_at integer not null,
    file_size  integer not null,
    file_hash  blob not null,

    -- extracted addon info
    title text not null,
    author text,
    version text,
    tagline text,
    chapter_ids text, -- comma separated list of coop chapter ids, if set
    flags integer not null default 0, -- Bit field

    -- misc
    workshop_id integer, -- extracted from addoninfo.txt or filename
    scan_id integer, -- for detecting missing files

    primary key (file_hash)
);

create table addon_tags
(
    hash blob not null,
    tag      text not null,
    primary key (hash, tag)
    foreign key (hash) references addons (hash)  ON UPDATE CASCADE ON DELETE CASCADE
);

create table workshop_items
(
    publishedfileid integer not null,
    title           text    not null,
    time_created    integer not null,
    time_updated    integer    null,
    file_size       integer not null,
    description     text    not null,
    file_url        text    not null,
    creator_id      text    not null,
    tags            text    not null, -- comma separate list

    src             text    not null, -- "workshop" folder or "addons" folder
    scan_id         integer null, -- for detecting missing files

    primary key(publishedfileid)
);


