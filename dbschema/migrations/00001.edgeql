CREATE MIGRATION m1rquzsix352jubbqnjgixnz4isirvm4ndbp5lsvszhxs5dkuk3j2a
    ONTO initial
{
  CREATE TYPE default::Inner {
      CREATE PROPERTY opt: std::str;
      CREATE REQUIRED PROPERTY req: std::str;
  };
  CREATE TYPE default::Outer {
      CREATE LINK inner: default::Inner;
      CREATE PROPERTY some_field: std::str;
  };
};
