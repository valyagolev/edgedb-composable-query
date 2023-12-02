CREATE MIGRATION m1mlmz4hdpjjtk2hxebie4painykgdga6epgpunxiqnpa46exzfrxq
    ONTO m1rquzsix352jubbqnjgixnz4isirvm4ndbp5lsvszhxs5dkuk3j2a
{
  ALTER TYPE default::Outer {
      CREATE LINK a: default::Inner;
      CREATE LINK b: default::Inner;
      CREATE REQUIRED PROPERTY other_field: std::str {
          SET REQUIRED USING (<std::str>{''});
      };
  };
};
