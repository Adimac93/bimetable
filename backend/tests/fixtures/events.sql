INSERT INTO events (id, owner_id, name, description, starts_at, ends_at)
VALUES
('6d185de5-ddec-462a-aeea-7628f03d417b', '29e40c2a-7595-42d3-98e8-9fe93ce99972', 'Matematyka', 'zadania optymalizacjne', '2023-03-07 08:00', '2023-03-07 09:35'),
('fd1dcdf7-de06-4aad-ba6e-f2097217a5b1', '29e40c2a-7595-42d3-98e8-9fe93ce99972', 'Fizyka', 'fizyka kwantowa :O', '2023-03-08 09:45', '2023-03-08 10:30'),
('d63a1036-e59d-4b7c-a009-9b90a0e703d1', 'a9c5900e-a445-4888-8612-4a5c8cadbd9e', 'Informatyka', NULL, '2023-03-07 11:40', '2023-03-07 13:15'),
('374ae0ab-d473-4752-b77f-cae55c69245c', '910e81a9-56df-4c24-965a-13eff739f469', 'Infa', NULL, '2023-03-07 11:30', '2023-03-07 13:15');

INSERT INTO recurrence_rules (event_id, recurrence, until, count, interval)
VALUES
('6d185de5-ddec-462a-aeea-7628f03d417b', '{"monthly": {"isByDay": true}}', '2024-01-07 9:35', 10, 1),
('fd1dcdf7-de06-4aad-ba6e-f2097217a5b1', '{"weekly": {"weekMap": 24}}', '2023-04-27 10:30', 15, 1),
('d63a1036-e59d-4b7c-a009-9b90a0e703d1', '{"weekly": {"weekMap": 40}}', '2023-04-27 13:15', 15, 1);
