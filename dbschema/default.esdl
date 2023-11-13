module default {
    type Inner {
        required req: str;
        opt: str;
    }
    type Outer {
        inner: Inner;
        required other_field: str;
        some_field: str;

        a: Inner;
        b: Inner;
    }
}
