BEGIN;

CREATE DOMAIN public.role
    AS text;

ALTER DOMAIN public.role OWNER TO postgres;

ALTER DOMAIN public.role
    ADD CONSTRAINT role_check CHECK (VALUE ~ 'owner'::text OR VALUE ~ 'editor'::text OR VALUE ~ 'reader'::text);


CREATE TABLE IF NOT EXISTS public.document_relation
(
    user_email character varying(50) COLLATE pg_catalog."default" NOT NULL,
    document_id character varying(100) COLLATE pg_catalog."default" NOT NULL,
    user_role role COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT document_relation_pkey PRIMARY KEY (user_email, document_id)
);

CREATE TABLE IF NOT EXISTS public.document_relation_group
(
    group_id serial NOT NULL,
    document_id character varying(100) COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT document_relation_group_pkey PRIMARY KEY (group_id, document_id)
);

CREATE TABLE IF NOT EXISTS public.group_members
(
    group_id serial NOT NULL,
    member_email character varying(100) COLLATE pg_catalog."default" NOT NULL
);

CREATE TABLE IF NOT EXISTS public.groups
(
    group_id serial NOT NULL,
    group_name character varying(100) COLLATE pg_catalog."default" NOT NULL,
    owner_email character varying(100) COLLATE pg_catalog."default" NOT NULL,
    group_role role COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT groups_pkey PRIMARY KEY (group_id),
    CONSTRAINT unique_group_name UNIQUE (owner_email, group_name)
);

CREATE TABLE IF NOT EXISTS public.users
(
    email character varying(50) COLLATE pg_catalog."default" NOT NULL,
    password character varying(50) COLLATE pg_catalog."default" NOT NULL,
    first_name character varying(50) COLLATE pg_catalog."default" NOT NULL,
    last_name character varying(50) COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT users_pkey PRIMARY KEY (email)
);

ALTER TABLE IF EXISTS public.document_relation
    ADD CONSTRAINT email_fk FOREIGN KEY (user_email)
    REFERENCES public.users (email) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.document_relation_group
    ADD CONSTRAINT group_id_fk FOREIGN KEY (group_id)
    REFERENCES public.groups (group_id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.group_members
    ADD CONSTRAINT group_id_fk FOREIGN KEY (group_id)
    REFERENCES public.groups (group_id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

ALTER TABLE IF EXISTS public.group_members
    ADD CONSTRAINT member_email_fk FOREIGN KEY (member_email)
    REFERENCES public.users (email) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.groups
    ADD CONSTRAINT owner_email_fk FOREIGN KEY (owner_email)
    REFERENCES public.users (email) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION;

ALTER TABLE IF EXISTS public.groups
    ADD CONSTRAINT unique_group_name
    UNIQUE (owner_email, group_name);



END;