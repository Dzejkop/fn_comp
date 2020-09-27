use applic::{applic, just_testing};

#[derive(Debug)]
pub struct Transaction<'a> {
    name: &'a str,
    counter: usize,
}

fn initiate_transaction<'a>(name: &'a str) -> Transaction<'a> {
    Transaction { name, counter: 0 }
}

fn inc_counter(mut transaction: Transaction) -> Transaction {
    transaction.counter += 1;
    transaction
}

fn validate_result(transaction: Transaction) -> Result<(Transaction, bool), ()> {
    if transaction.counter > 0 {
        Ok((transaction, true))
    } else {
        Err(())
    }
}

fn finalize(transaction: Transaction, is_valid: bool) -> Result<Transaction, ()> {
    if is_valid {
        Ok(transaction)
    } else {
        Err(())
    }
}

fn main() {
    let procedure = applic!(
        initiate_transaction(_)
        => inc_counter(_)
        => inc_counter(_)
        => inc_counter(_)
        => validate_result(_)?
        => finalize(_, _)
        => Result::unwrap(_)
    );
    applic!();

    // let x = just_testing!(|n| 123);

    // let closure_procedure = applic!(
    //     |n| n + 1
    //     => |n| n * 2
    //     => |n| (n / 2, n % 2)
    //     => |a, b| a + b
    // );

    // fn equivalent(n: i32) -> i32 {
    //     let n = n + 1;
    //     let n = n * 2;
    //     let (a, b) = (n / 2, n % 2);
    //     a + b
    // }

    // assert_eq!(equivalent(10), closure_procedure(10));

    let transaction = procedure("hello");

    println!("{:?}", transaction);
}
