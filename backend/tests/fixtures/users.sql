INSERT INTO users (id, username, tag)
VALUES
('32190025-7c15-4adb-82fd-9acc3dc8e7b6','mabi19', 0000),
('29e40c2a-7595-42d3-98e8-9fe93ce99972','pkb-pmj', 0000),
('910e81a9-56df-4c24-965a-13eff739f469','adimac93', 0000),
('a9c5900e-a445-4888-8612-4a5c8cadbd9e','hubertk', 0000);

-- password #strong#_#pass#
INSERT INTO credentials (user_id, login, password)
VALUES
('32190025-7c15-4adb-82fd-9acc3dc8e7b6','mabmab','$argon2i$v=19$m=4096,t=3,p=1$M0g3ODVzWmQ$fHLpcolZURzJzej/xbDQqTb+OINmUOl8uEFVLah0z8Y'),
('29e40c2a-7595-42d3-98e8-9fe93ce99972','pkbpkp','$argon2i$v=19$m=4096,t=3,p=1$M0g3ODVzWmQ$fHLpcolZURzJzej/xbDQqTb+OINmUOl8uEFVLah0z8Y'),
('910e81a9-56df-4c24-965a-13eff739f469','macmac','$argon2i$v=19$m=4096,t=3,p=1$M0g3ODVzWmQ$fHLpcolZURzJzej/xbDQqTb+OINmUOl8uEFVLah0z8Y'),
('a9c5900e-a445-4888-8612-4a5c8cadbd9e','hubhub','$argon2i$v=19$m=4096,t=3,p=1$M0g3ODVzWmQ$fHLpcolZURzJzej/xbDQqTb+OINmUOl8uEFVLah0z8Y');
