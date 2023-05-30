CREATE TABLE IF NOT EXISTS user_folder 
    (
        tag_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, 
        tag_name TEXT NOT NULL,
        owner_id TEXT NOT NULL,
        bot BOOLEAN NOT NULL DEFAULT false,
        valid_count INTEGER NOT NULL DEFAULT 0,
        UNIQUE (tag_name, owner_id)
    );
CREATE TABLE IF NOT EXISTS dist 
    (
        user_id TEXT NOT NULL,
        tag_id INTEGER NOT NULL, 
        dist_channel_id TEXT NOT NULL,
        PRIMARY KEY(user_id, tag_id, dist_channel_id),
        FOREIGN KEY (tag_id) REFERENCES user_folder(tag_id) ON DELETE CASCADE
    );
CREATE TABLE IF NOT EXISTS channel_list
    (
        tag_id INTEGER NOT NULL, 
        channel_id TEXT NOT NULL,
        PRIMARY KEY(tag_id, channel_id),
        FOREIGN KEY (tag_id) REFERENCES user_folder(tag_id) ON DELETE CASCADE
    );

INSERT INTO user_folder (tag_name, owner_id) VALUES ("test_a", "U00001");
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C01' FROM user_folder WHERE tag_name = 'test_a' AND owner_id = 'U00001';
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C02' FROM user_folder WHERE tag_name = 'test_a' AND owner_id = 'U00001';


INSERT INTO user_folder (tag_name, owner_id) VALUES ("test_b", "U00001");
INSERT INTO user_folder (tag_name, owner_id) VALUES ("test_pub", "public");
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C03' FROM user_folder WHERE tag_name = 'test_pub' AND owner_id = 'public';


INSERT INTO user_folder (tag_name, owner_id) VALUES ("test_dist", "U0987654");
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C0123456789' FROM user_folder WHERE tag_name = 'test_dist' AND owner_id = 'U0987654';

INSERT INTO user_folder (tag_name, owner_id) VALUES ("test_target", "U00001");
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C01' FROM user_folder WHERE tag_name = 'test_target' AND owner_id = 'U00001';
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C02' FROM user_folder WHERE tag_name = 'test_target' AND owner_id = 'U00001';
INSERT INTO dist  (tag_id,user_id, dist_channel_id)
    SELECT tag_id, 'U00001', 'Cdist' FROM user_folder WHERE tag_name = 'test_target' AND owner_id = 'U00001';



INSERT INTO user_folder (tag_name, owner_id, bot) VALUES ("test_target_bot", "U00001", true);
INSERT INTO channel_list (tag_id, channel_id)
    SELECT tag_id, 'C02' FROM user_folder WHERE tag_name = 'test_target_bot' AND owner_id = 'U00001';
INSERT INTO dist  (tag_id,user_id ,dist_channel_id)
    SELECT tag_id, 'U00001', 'Cdist_bot' FROM user_folder WHERE tag_name = 'test_target_bot' AND owner_id = 'U00001';


UPDATE user_folder
    SET valid_count = valid_count + 1
    WHERE tag_name = 'test_target';
UPDATE user_folder
    SET valid_count = valid_count + 1
    WHERE tag_name = 'test_target_bot';