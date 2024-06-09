CREATE TABLE cases (
    id SERIAL PRIMARY KEY,
    read_at TIMESTAMP,
    npi VARCHAR(50),
    exam_name VARCHAR(50),
    modality VARCHAR(50),
    subspecialty VARCHAR(50),
    is_child BOOLEAN,
    facility_name VARCHAR(50)
);