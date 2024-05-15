DEFINE TABLE User SCHEMAFULL CHANGEFEED 50w INCLUDE ORIGINAL;

DEFINE FIELD email ON TABLE User TYPE string
  ASSERT string::is::email($value);
DEFINE INDEX user_email_unique_index ON TABLE User COLUMNS email UNIQUE;

DEFINE FIELD password ON TABLE User TYPE option<string>;
DEFINE FIELD created_at ON User TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON User TYPE datetime VALUE time::now();

DEFINE FIELD roles ON TABLE User TYPE array
  ASSERT array::len($value) > 0;
DEFINE FIELD roles.* ON TABLE User TYPE string;
DEFINE INDEX roles_index ON TABLE User COLUMNS roles.*;

