#[cfg(feature = "alloc")]
mod alloc;
mod core;
mod rend;
#[cfg(feature = "std")]
mod std;

// Support for various common crates. These are primarily to get users off the
// ground and build some momentum.

// These are NOT PLANNED to remain in rkyv for the final release. Much like
// serde, these implementations should be moved into their respective crates
// over time. Before adding support for another crate, please consider getting
// rkyv support in the crate instead.

#[cfg(feature = "arrayvec")]
mod arrayvec;
#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "hashbrown")]
mod hashbrown;
#[cfg(feature = "indexmap")]
mod indexmap;
#[cfg(feature = "smallvec")]
mod smallvec;
#[cfg(feature = "smol_str")]
mod smolstr;
#[cfg(feature = "thin-vec")]
mod thin_vec;
#[cfg(feature = "tinyvec")]
mod tinyvec;
#[cfg(feature = "triomphe")]
mod triomphe;
#[cfg(feature = "uuid")]
mod uuid;

#[cfg(test)]
mod core_tests {
    use core::pin::Pin;

    use rancor::{Fallible, Source};

    use crate::{
        api::test::{roundtrip, to_archived},
        option::ArchivedOption,
        primitive::{ArchivedI32, ArchivedU32},
        Archive, Deserialize, Place, Portable, Serialize,
    };

    #[test]
    fn roundtrip_unit_struct() {
        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq), derive(Debug))]
        struct Test;

        roundtrip(&Test);
        roundtrip(&[Test, Test]);
    }

    #[test]
    fn roundtrip_tuple_struct() {
        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq), derive(Debug))]
        struct Test((), i32, Option<i32>);

        roundtrip(&Test((), 42, Some(42)));
        roundtrip(&[Test((), 42, Some(42)), Test((), 42, Some(42))]);
    }

    #[test]
    fn roundtrip_struct() {
        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq), derive(Debug))]
        struct Test {
            a: (),
            b: i32,
            c: Option<i32>,
        }

        roundtrip(&Test {
            a: (),
            b: 42,
            c: Some(42),
        });
        roundtrip(&[
            Test {
                a: (),
                b: 42,
                c: Some(42),
            },
            Test {
                a: (),
                b: 42,
                c: Some(42),
            },
        ]);
    }

    #[test]
    fn roundtrip_generic_struct() {
        use core::fmt;

        pub trait TestTrait {
            type Associated: PartialEq;
        }

        impl TestTrait for () {
            type Associated = i32;
        }

        #[derive(Archive, Serialize, Deserialize, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq))]
        struct Test<T: TestTrait> {
            a: (),
            b: <T as TestTrait>::Associated,
            c: Option<i32>,
        }

        impl<T: TestTrait> fmt::Debug for Test<T>
        where
            T::Associated: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("Test")
                    .field("a", &self.a)
                    .field("b", &self.b)
                    .field("c", &self.c)
                    .finish()
            }
        }

        impl<T: TestTrait> fmt::Debug for ArchivedTest<T>
        where
            T::Associated: Archive,
            <T::Associated as Archive>::Archived: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("Test")
                    .field("a", &self.a)
                    .field("b", &self.b)
                    .field("c", &self.c)
                    .finish()
            }
        }

        roundtrip(&Test::<()> {
            a: (),
            b: 42,
            c: Some(42),
        });
        roundtrip(&[
            Test::<()> {
                a: (),
                b: 42,
                c: Some(42),
            },
            Test::<()> {
                a: (),
                b: 42,
                c: Some(42),
            },
        ]);
    }

    #[test]
    fn roundtrip_enum() {
        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq), derive(Debug))]
        enum Test {
            A,
            B(i32),
            C { inner: i32 },
        }

        roundtrip(&Test::A);
        roundtrip(&Test::B(42));
        roundtrip(&Test::C { inner: 42 });
        roundtrip(&[Test::A, Test::B(42), Test::C { inner: 42 }]);
    }

    #[test]
    fn roundtrip_generic_enum() {
        use core::fmt;

        pub trait TestTrait {
            type Associated: PartialEq;
        }

        impl TestTrait for () {
            type Associated = i32;
        }

        #[derive(Archive, Serialize, Deserialize, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq))]
        enum Test<T: TestTrait> {
            A,
            B(<T as TestTrait>::Associated),
            C { inner: <T as TestTrait>::Associated },
        }

        impl<T: TestTrait> fmt::Debug for Test<T>
        where
            T::Associated: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Test::A => f.debug_tuple("Test::A").finish(),
                    Test::B(value) => {
                        f.debug_tuple("Test::B").field(value).finish()
                    }
                    Test::C { inner } => {
                        f.debug_struct("Test::C").field("inner", inner).finish()
                    }
                }
            }
        }

        impl<T: TestTrait> fmt::Debug for ArchivedTest<T>
        where
            T::Associated: Archive,
            <T::Associated as Archive>::Archived: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    ArchivedTest::A => {
                        f.debug_tuple("ArchivedTest::A").finish()
                    }
                    ArchivedTest::B(value) => {
                        f.debug_tuple("ArchivedTest::B").field(value).finish()
                    }
                    ArchivedTest::C { inner } => f
                        .debug_struct("ArchivedTest::C")
                        .field("inner", inner)
                        .finish(),
                }
            }
        }

        roundtrip(&Test::<()>::A);
        roundtrip(&Test::<()>::B(42));
        roundtrip(&Test::<()>::C { inner: 42 });
        roundtrip(&[
            Test::<()>::A,
            Test::<()>::B(42),
            Test::<()>::C { inner: 42 },
        ]);
    }

    #[test]
    fn basic_mutable_refs() {
        to_archived(&42i32, |mut archived| {
            assert_eq!(*archived, 42);
            *archived = 11.into();
            assert_eq!(*archived, 11);
        });
    }

    #[test]
    fn struct_mutable_refs() {
        #[derive(Archive, Serialize)]
        #[rkyv(crate, check_bytes)]
        struct Opaque(i32);

        impl ArchivedOpaque {
            fn get(&self) -> i32 {
                self.0.into()
            }

            fn set(mut self: Pin<&mut Self>, value: i32) {
                (*self).0 = value.into();
            }
        }

        #[derive(Archive, Serialize)]
        #[rkyv(crate, check_bytes)]
        struct Test {
            a: Opaque,
        }

        impl ArchivedTest {
            fn a(self: Pin<&mut Self>) -> Pin<&mut ArchivedOpaque> {
                unsafe { self.map_unchecked_mut(|s| &mut s.a) }
            }
        }

        let value = Test { a: Opaque(10) };

        to_archived(&value, |mut archived| {
            assert_eq!(archived.a.get(), 10);

            archived.as_mut().a().set(50);
            assert_eq!(archived.a.get(), 50);
        })
    }

    #[test]
    fn enum_mutable_ref() {
        #[allow(dead_code)]
        #[derive(Archive, Serialize)]
        #[rkyv(crate, check_bytes)]
        enum Test {
            A,
            B(char),
            C(i32),
        }

        let value = Test::A;

        to_archived(&value, |mut archived| {
            if let ArchivedTest::A = *archived {
                ()
            } else {
                panic!("incorrect enum after archiving");
            }

            *archived = ArchivedTest::C(42.into());

            if let ArchivedTest::C(i) = *archived {
                assert_eq!(i, 42);
            } else {
                panic!("incorrect enum after mutation");
            }
        });
    }

    #[test]
    fn complex_bounds() {
        use core::marker::PhantomData;

        trait MyTrait {}

        impl MyTrait for i32 {}

        #[derive(Portable)]
        #[rkyv(crate)]
        #[repr(transparent)]
        struct MyStruct<T> {
            _phantom: PhantomData<T>,
        }

        impl<T: Archive + MyTrait> Archive for MyStruct<T> {
            type Archived = MyStruct<T::Archived>;
            type Resolver = ();

            fn resolve(&self, _: Self::Resolver, _: Place<Self::Archived>) {}
        }

        impl<T, S> Serialize<S> for MyStruct<T>
        where
            T: Archive + MyTrait,
            S: Fallible + MyTrait + ?Sized,
        {
            fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
                Ok(())
            }
        }

        impl<T, D> Deserialize<MyStruct<T>, D> for MyStruct<T::Archived>
        where
            T: Archive + MyTrait,
            D: Fallible + MyTrait + ?Sized,
        {
            fn deserialize(&self, _: &mut D) -> Result<MyStruct<T>, D::Error> {
                Ok(MyStruct {
                    _phantom: PhantomData,
                })
            }
        }

        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(
            crate,
            archive_bounds(T: MyTrait),
            serialize_bounds(__S: MyTrait),
            deserialize_bounds(__D: MyTrait),
        )]
        enum Node<T> {
            Nil,
            Cons {
                value: T,
                #[omit_bounds]
                next: MyStruct<Self>,
            },
        }

        impl<T: MyTrait> MyTrait for Node<T> {}
    }

    #[test]
    fn derive_attributes() {
        #[derive(Archive, Debug, PartialEq)]
        #[rkyv(
            crate,
            archived = ATest,
            resolver = RTest,
            check_bytes,
            compare(PartialEq),
            derive(Debug),
        )]
        struct Test {
            a: i32,
            b: Option<u32>,
        }

        impl<S> Serialize<S> for Test
        where
            S: Fallible + ?Sized,
            S::Error: Source,
            i32: Serialize<S>,
            Option<u32>: Serialize<S>,
        {
            fn serialize(&self, serializer: &mut S) -> Result<RTest, S::Error> {
                Ok(RTest {
                    a: self.a.serialize(serializer)?,
                    b: self.b.serialize(serializer)?,
                })
            }
        }

        impl<D> Deserialize<Test, D> for ATest
        where
            D: Fallible + ?Sized,
            D::Error: Source,
            ArchivedI32: Deserialize<i32, D>,
            ArchivedOption<ArchivedU32>: Deserialize<Option<u32>, D>,
        {
            fn deserialize(
                &self,
                deserializer: &mut D,
            ) -> Result<Test, D::Error> {
                Ok(Test {
                    a: self.a.deserialize(deserializer)?,
                    b: self.b.deserialize(deserializer)?,
                })
            }
        }

        let value = Test { a: 42, b: Some(12) };

        roundtrip(&value);
    }

    #[test]
    fn compare() {
        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate, compare(PartialEq, PartialOrd))]
        pub struct UnitFoo;

        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate, compare(PartialEq, PartialOrd))]
        pub struct TupleFoo(i32);

        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate, compare(PartialEq, PartialOrd))]
        pub struct StructFoo {
            t: i32,
        }

        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate, compare(PartialEq, PartialOrd))]
        pub enum EnumFoo {
            #[allow(dead_code)]
            Foo(i32),
        }
    }

    #[test]
    fn default_type_parameters() {
        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate)]
        pub struct TupleFoo<T = i32>(T);

        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate)]
        pub struct StructFoo<T = i32> {
            t: T,
        }

        #[derive(Archive, Serialize, Deserialize)]
        #[rkyv(crate)]
        pub enum EnumFoo<T = i32> {
            #[allow(dead_code)]
            T(T),
        }
    }

    #[test]
    fn const_generics() {
        #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
        #[rkyv(crate, check_bytes, compare(PartialEq), derive(Debug))]
        pub struct Const<const N: usize>;

        roundtrip(&Const::<1>);
        roundtrip(&Const::<2>);
        roundtrip(&Const::<3>);

        #[derive(Archive, Deserialize, Serialize)]
        #[rkyv(crate)]
        pub struct Array<T, const N: usize>([T; N]);
    }

    #[test]
    fn repr_c_packed() {
        #[derive(Archive)]
        #[rkyv(crate, attr(repr(C, packed)))]
        #[allow(dead_code)]
        struct CPackedRepr {
            a: u8,
            b: u32,
            c: u8,
        }

        assert_eq!(core::mem::size_of::<ArchivedCPackedRepr>(), 6);

        #[derive(Archive)]
        #[rkyv(crate, attr(repr(C), repr(packed)))]
        #[allow(dead_code)]
        struct CPackedRepr2 {
            a: u8,
            b: u32,
            c: u8,
        }

        assert_eq!(core::mem::size_of::<ArchivedCPackedRepr2>(), 6);
    }

    #[test]
    fn repr_c_align() {
        #[derive(Archive)]
        #[rkyv(crate, attr(repr(C, align(8))))]
        #[allow(dead_code)]
        struct CAlignRepr {
            a: u8,
        }

        assert_eq!(core::mem::align_of::<ArchivedCAlignRepr>(), 8);

        #[derive(Archive)]
        #[rkyv(crate, attr(repr(C), repr(align(8))))]
        #[allow(dead_code)]
        struct CAlignRepr2 {
            a: u8,
        }

        assert_eq!(core::mem::align_of::<ArchivedCAlignRepr>(), 8);
    }

    #[test]
    fn archive_as_unit_struct() {
        #[derive(
            Archive, Serialize, Deserialize, Debug, Portable, PartialEq,
        )]
        #[cfg_attr(feature = "bytecheck", derive(bytecheck::CheckBytes))]
        #[rkyv(crate, as = "ExampleUnitStruct")]
        #[repr(C)]
        struct ExampleUnitStruct;

        roundtrip(&ExampleUnitStruct);
    }

    #[test]
    fn archive_as_tuple_struct() {
        #[derive(Archive, Serialize, Deserialize, Debug, Portable)]
        #[cfg_attr(feature = "bytecheck", derive(bytecheck::CheckBytes))]
        #[rkyv(crate, as = "ExampleTupleStruct<T::Archived>")]
        #[repr(transparent)]
        struct ExampleTupleStruct<T>(T);

        impl<T: PartialEq<U>, U> PartialEq<ExampleTupleStruct<U>>
            for ExampleTupleStruct<T>
        {
            fn eq(&self, other: &ExampleTupleStruct<U>) -> bool {
                self.0.eq(&other.0)
            }
        }

        roundtrip(&ExampleTupleStruct(42i32));
    }

    #[test]
    fn archive_as_struct() {
        #[derive(Archive, Serialize, Deserialize, Debug, Portable)]
        #[cfg_attr(feature = "bytecheck", derive(bytecheck::CheckBytes))]
        #[rkyv(crate, as = "ExampleStruct<T::Archived>")]
        #[repr(transparent)]
        struct ExampleStruct<T> {
            value: T,
        }

        impl<T, U> PartialEq<ExampleStruct<U>> for ExampleStruct<T>
        where
            T: PartialEq<U>,
        {
            fn eq(&self, other: &ExampleStruct<U>) -> bool {
                self.value.eq(&other.value)
            }
        }

        roundtrip(&ExampleStruct { value: 42i32 });
    }

    #[test]
    fn archive_as_enum() {
        #[derive(Archive, Serialize, Deserialize, Debug, Portable)]
        #[cfg_attr(feature = "bytecheck", derive(bytecheck::CheckBytes))]
        #[rkyv(crate, as = "ExampleEnum<T::Archived>")]
        #[repr(u8)]
        enum ExampleEnum<T> {
            A(T),
            B,
        }

        impl<T: PartialEq<U>, U> PartialEq<ExampleEnum<U>> for ExampleEnum<T> {
            fn eq(&self, other: &ExampleEnum<U>) -> bool {
                match self {
                    ExampleEnum::A(value) => {
                        if let ExampleEnum::A(other) = other {
                            value.eq(other)
                        } else {
                            false
                        }
                    }
                    ExampleEnum::B => {
                        if let ExampleEnum::B = other {
                            true
                        } else {
                            false
                        }
                    }
                }
            }
        }

        roundtrip(&ExampleEnum::A(42i32));
    }

    #[test]
    fn archive_crate_path() {
        use crate as alt_path;

        #[derive(Archive, Deserialize, Serialize)]
        #[rkyv(crate = alt_path)]
        struct Test<'a> {
            #[with(alt_path::with::InlineAsBox)]
            value: &'a str,
            other: i32,
        }
    }

    #[test]
    fn pass_thru_derive_with_option() {
        #[derive(
            Clone, Copy, Debug, PartialEq, Archive, Serialize, Deserialize,
        )]
        #[rkyv(crate, compare(PartialEq), derive(Clone, Copy, Debug))]
        enum ExampleEnum {
            Foo,
            Bar(u64),
        }

        #[derive(
            Clone, Copy, Debug, PartialEq, Archive, Serialize, Deserialize,
        )]
        #[rkyv(crate, compare(PartialEq), derive(Clone, Copy, Debug))]
        struct Example {
            x: i32,
            y: Option<ExampleEnum>,
        }

        let _ = Example {
            x: 0,
            y: Some(ExampleEnum::Bar(0)),
        };
    }
}

#[cfg(all(test, feature = "alloc"))]
mod alloc_tests {
    use core::pin::Pin;

    use rancor::Source;

    use crate::{
        alloc::{
            boxed::Box,
            string::{String, ToString},
            vec,
            vec::Vec,
        },
        api::test::{roundtrip, to_archived},
        primitive::ArchivedI32,
        ser::Writer,
        Archive, Deserialize, Serialize,
    };

    #[test]
    fn struct_container_mutable_refs() {
        use crate::{
            boxed::ArchivedBox, string::ArchivedString, vec::ArchivedVec,
        };

        #[derive(Archive, Serialize)]
        #[rkyv(crate, check_bytes)]
        struct Test {
            a: Box<i32>,
            b: Vec<String>,
        }

        impl ArchivedTest {
            fn a(self: Pin<&mut Self>) -> Pin<&mut ArchivedBox<ArchivedI32>> {
                unsafe { self.map_unchecked_mut(|s| &mut s.a) }
            }

            fn b(
                self: Pin<&mut Self>,
            ) -> Pin<&mut ArchivedVec<ArchivedString>> {
                unsafe { self.map_unchecked_mut(|s| &mut s.b) }
            }
        }

        let value = Test {
            a: Box::new(10),
            b: vec!["hello".to_string(), "world".to_string()],
        };

        to_archived(&value, |mut archived| {
            assert_eq!(*archived.a, 10);
            assert_eq!(archived.b.len(), 2);
            assert_eq!(archived.b[0], "hello");
            assert_eq!(archived.b[1], "world");

            *archived.as_mut().a().get_pin() = 50.into();
            assert_eq!(*archived.a, 50);

            archived
                .as_mut()
                .b()
                .index_pin(0)
                .as_pin_str()
                .make_ascii_uppercase();
            archived
                .as_mut()
                .b()
                .index_pin(1)
                .as_pin_str()
                .make_ascii_uppercase();
            assert_eq!(archived.b[0], "HELLO");
            assert_eq!(archived.b[1], "WORLD");
        });
    }

    #[test]
    fn recursive_structures() {
        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[rkyv(
            crate,
            check_bytes(bounds(__C: crate::validation::ArchiveContext)),
            // The derive macros don't apply the right bounds from Box so we
            // have to manually specify what bounds to apply
            serialize_bounds(__S: Writer),
            deserialize_bounds(__D::Error: Source),
            compare(PartialEq),
            derive(Debug),
        )]
        enum Node {
            Nil,
            Cons(#[omit_bounds] Box<Node>),
        }

        roundtrip(&Node::Cons(Box::new(Node::Cons(Box::new(Node::Nil)))));
    }

    #[test]
    fn recursive_self_types() {
        #[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
        #[rkyv(
            crate,
            check_bytes(bounds(__C: crate::validation::ArchiveContext)),
            archive_bounds(T::Archived: core::fmt::Debug),
            // The derive macros don't apply the right bounds from Box so we
            // have to manually specify what bounds to apply
            serialize_bounds(__S: Writer),
            deserialize_bounds(__D::Error: Source),
            compare(PartialEq),
            derive(Debug),
        )]
        pub enum LinkedList<T: Archive> {
            Empty,
            Node {
                val: T,
                #[omit_bounds]
                next: Box<Self>,
            },
        }

        roundtrip(&LinkedList::Node {
            val: 42i32,
            next: Box::new(LinkedList::Node {
                val: 100i32,
                next: Box::new(LinkedList::Empty),
            }),
        });
    }
}
