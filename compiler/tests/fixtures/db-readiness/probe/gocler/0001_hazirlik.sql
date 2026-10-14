CREATE TABLE public.zee_app_state (
    singleton boolean PRIMARY KEY CHECK (singleton),
    revision text NOT NULL
);
INSERT INTO public.zee_app_state VALUES (true, 'readiness-v1');
