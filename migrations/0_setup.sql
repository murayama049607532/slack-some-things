CREATE TABLE IF NOT EXISTS user_folder 
    (
        tag_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, 
        tag_name TEXT NOT NULL,
        owner_id TEXT NOT NULL,
        bot BOOLEAN NOT NULL DEFAULT false,
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
INSERT INTO user_folder (tag_name, owner_id) VALUES ("test", "U01234567");
