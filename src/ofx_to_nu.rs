use chrono::DateTime;
use nu_protocol::{Span, Value, record};
use ofx_rs::{
    OfxDocument,
    aggregates::{
        AvailableBalance, Balance, BankAccount, CcStatementResponse, CreditCardAccount,
        CurrencyInfo, LedgerBalance, Payee, StatementResponse, StatementTransaction, Status,
        TransactionList, TransactionWrapper,
    },
    document::{BankingMessageSet, CreditCardMessageSet, SignonResponse},
    header::OfxHeader,
    types::{OfxAmount, OfxDateTime},
};

pub fn convert_document(doc: &OfxDocument, span: Span) -> Value {
    Value::record(
        record! {
            "header" => convert_header(doc.header(), span),
            "signon" => convert_signon(doc.signon(), span),
            "banking" => doc.banking().map_or(Value::nothing(span), |b| convert_banking(b, span)),
            "credit_card" => doc.credit_card().map_or(Value::nothing(span), |c| convert_credit_card(c, span)),
        },
        span,
    )
}

fn convert_header(h: &OfxHeader, span: Span) -> Value {
    Value::record(
        record! {
            "version" => Value::string(format!("{}", h.version()), span),
            "security" => Value::string(h.security().to_string(), span),
            "old_file_uid" => Value::string(h.old_file_uid(), span),
            "new_file_uid" => Value::string(h.new_file_uid(), span),
        },
        span,
    )
}

fn convert_signon(s: &SignonResponse, span: Span) -> Value {
    Value::record(
        record! {
            "status" => convert_status(s.status(), span),
            "date_time_server" => convert_datetime(s.date_time_server(), span),
            "language" => Value::string(s.language(), span),
            "date_time_profup" => s.date_time_profup().map_or(Value::nothing(span), |d| convert_datetime(d, span)),
            "date_time_acctup" => s.date_time_acctup().map_or(Value::nothing(span), |d| convert_datetime(d, span)),
            "fi_org" => s.fi_org().map_or(Value::nothing(span), |x| Value::string(x, span)),
            "fi_id" => s.fi_id().map_or(Value::nothing(span), |x| Value::string(x, span)),
            "session_cookie" => s.session_cookie().map_or(Value::nothing(span), |x| Value::string(x, span)),
            "access_key" => s.access_key().map_or(Value::nothing(span), |x| Value::string(x, span)),
        },
        span,
    )
}

fn convert_banking(set: &BankingMessageSet, span: Span) -> Value {
    Value::list(
        set.statement_responses()
            .iter()
            .map(|res| convert_wrapper(res, span))
            .collect(),
        span,
    )
}

fn convert_wrapper(res: &TransactionWrapper<StatementResponse>, span: Span) -> Value {
    Value::record(
        record! {
            "status" => convert_status(res.status(), span),
            "response" => res.response().map_or(Value::nothing(span), |r| convert_response(r, span)),
        },
        span,
    )
}

fn convert_status(s: &Status, span: Span) -> Value {
    Value::record(
        record! {
            "code" => Value::int(s.code().into(), span),
            "severity" => Value::string(s.severity().to_string(), span),
            "message" => s.message().map_or(Value::nothing(span), |m| Value::string(m, span)),
        },
        span,
    )
}

fn convert_response(r: &StatementResponse, span: Span) -> Value {
    Value::record(
        record! {
            "currency_default" => Value::string(r.currency_default().as_str(), span),
            "bank_account" => convert_bank_account(r.bank_account(), span),
            "ledger_balance" => r.ledger_balance().map_or(Value::nothing(span), |b| convert_ledger_balance(b, span)),
            "available_balance" => r.available_balance().map_or(Value::nothing(span), |b| convert_available_balance(b, span)),
            "balance_list" => Value::list(r.balance_list().iter().map(|b| convert_balance(b, span)).collect(), span),
            "transaction_list" => r.transaction_list().map_or(Value::nothing(span), |l| convert_transaction_list(l, span)),
            "marketing_info" => r.marketing_info().map_or(Value::nothing(span), |m| Value::string(m, span)),
        },
        span,
    )
}

// Credit Card

fn convert_credit_card(set: &CreditCardMessageSet, span: Span) -> Value {
    Value::list(
        set.statement_responses()
            .iter()
            .map(|res| convert_wrapper_cc(res, span))
            .collect(),
        span,
    )
}

fn convert_wrapper_cc(res: &TransactionWrapper<CcStatementResponse>, span: Span) -> Value {
    Value::record(
        record! {
            "status" => convert_status(res.status(), span),
            "response" => res.response().map_or(Value::nothing(span), |r| convert_response_cc(r, span)),
        },
        span,
    )
}

fn convert_response_cc(r: &CcStatementResponse, span: Span) -> Value {
    Value::record(
        record! {
            "currency_default" => Value::string(r.currency_default().as_str(), span),
            "credit_card_account" => convert_cc_account(r.credit_card_account(), span),
            "ledger_balance" => r.ledger_balance().map_or(Value::nothing(span), |b| convert_ledger_balance(b, span)),
            "available_balance" => r.available_balance().map_or(Value::nothing(span), |b| convert_available_balance(b, span)),
            "balance_list" => Value::list(r.balance_list().iter().map(|b| convert_balance(b, span)).collect(), span),
            "transaction_list" => r.transaction_list().map_or(Value::nothing(span), |l| convert_transaction_list(l, span)),
            "marketing_info" => r.marketing_info().map_or(Value::nothing(span), |m| Value::string(m, span)),
        },
        span,
    )
}
// ---

fn convert_bank_account(a: &BankAccount, span: Span) -> Value {
    Value::record(
        record! {
            "bank" => Value::string(a.bank_id().as_str(), span),
            "branch" => a.branch_id().map_or(Value::nothing(span), |b| Value::string(b.as_str(), span)),
            "account" => Value::string(a.account_id().as_str(), span),
            "account_type" => Value::string(a.account_type().to_string(), span),
            "account_key" => a.account_key().map_or(Value::nothing(span), |k| Value::string(k, span)),
        },
        span,
    )
}

fn convert_cc_account(a: &CreditCardAccount, span: Span) -> Value {
    Value::record(
        record! {
            "account_id" => Value::string(a.account_id().as_str(), span),
            "account_key" => a.account_key().map_or(Value::nothing(span), |k| Value::string(k, span)),
        },
        span,
    )
}

fn convert_balance(b: &Balance, span: Span) -> Value {
    Value::record(
        record! {
            "name" => Value::string(b.name(), span),
            "description" => Value::string(b.description(), span),
            "kind" => Value::string(b.kind().to_string(), span),
            "value" => convert_amount(&b.value(), span),
            "as_of" => b.as_of().map_or(Value::nothing(span), |d| convert_datetime(d, span)),
            "currency" => b.currency().map_or(Value::nothing(span), |c| Value::string(c.as_str(), span)),
        },
        span,
    )
}

fn convert_ledger_balance(b: &LedgerBalance, span: Span) -> Value {
    Value::record(
        record! {
            "amount" => convert_amount(&b.amount(), span),
            "as_of" => convert_datetime(b.as_of(), span),
        },
        span,
    )
}

fn convert_available_balance(b: &AvailableBalance, span: Span) -> Value {
    Value::record(
        record! {
            "amount" => convert_amount(&b.amount(), span),
            "as_of" => convert_datetime(b.as_of(), span),
        },
        span,
    )
}

fn convert_transaction_list(l: &TransactionList, span: Span) -> Value {
    Value::record(
        record! {
            "start" => convert_datetime(l.start(), span),
            "end" => convert_datetime(l.end(), span),
            "transactions" => Value::list(l.transactions().iter().map(|t| convert_transaction(t, span)).collect(), span), // Vec<StatementTransaction>
        },
        span,
    )
}

fn convert_transaction(t: &StatementTransaction, span: Span) -> Value {
    Value::record(
        record! {
            "transaction_type" => Value::string(t.transaction_type().to_string(), span),
            "date_posted" => convert_datetime(t.date_posted(), span),
            "date_user" => t.date_user().map_or(Value::nothing(span), |d| convert_datetime(d, span)),
            "date_available" => t.date_available().map_or(Value::nothing(span), |d| convert_datetime(d, span)),
            "amount" => convert_amount(&t.amount(), span),
            "fit_id" => Value::string(t.fit_id().as_str(), span),
            "correction_id" => t.correction_id().map_or(Value::nothing(span), |i| Value::string(i.as_str(), span)),
            "correction_action" => t.correction_action().map_or(Value::nothing(span), |a| Value::string(a.to_string(), span)),
            "server_transaction_id" => t.server_transaction_id().map_or(Value::nothing(span), |i| Value::string(i.as_str(), span)),
            "check_number" => t.check_number().map_or(Value::nothing(span), |n| Value::string(n.as_str(), span)),
            "reference_number" => t.reference_number().map_or(Value::nothing(span), |n| Value::string(n, span)),
            "sic" => t.sic().map_or(Value::nothing(span), |c| Value::int(c.into(), span)),
            "payee_id" => t.payee_id().map_or(Value::nothing(span), |n| Value::string(n, span)),
            "name" => t.name().map_or(Value::nothing(span), |n| Value::string(n, span)),
            "payee" => t.payee().map_or(Value::nothing(span), |p| convert_payee(p, span)),
            "bank_account_to" => t.bank_account_to().map_or(Value::nothing(span), |a| convert_bank_account(a, span)),
            "cc_account_to" => t.cc_account_to().map_or(Value::nothing(span), |a| convert_cc_account(a, span)),
            "memo" => t.memo().map_or(Value::nothing(span), |m| Value::string(m, span)),
            "currency" => t.currency().map_or(Value::nothing(span), |c| convert_currency(c, span)),
            "inv401k_source" => t.inv401k_source().map_or(Value::nothing(span), |s| Value::string(s.to_string(), span)),
        },
        span,
    )
}

fn convert_payee(p: &Payee, span: Span) -> Value {
    Value::record(
        record! {
            "name" => Value::string(p.name(), span),
            "address1" => Value::string(p.address1(), span),
            "address2" => p.address2().map_or(Value::nothing(span), |a| Value::string(a, span)),
            "address3" => p.address3().map_or(Value::nothing(span), |a| Value::string(a, span)),
            "city" => Value::string(p.city(), span),
            "state" => Value::string(p.state(), span),
            "postal_code" => Value::string(p.postal_code(), span),
            "country" => p.country().map_or(Value::nothing(span), |c| Value::string(c, span)),
            "phone" => Value::string(p.phone(), span),
        },
        span,
    )
}

fn convert_currency(c: &CurrencyInfo, span: Span) -> Value {
    let orig = match c {
        CurrencyInfo::Currency { code: _, rate: _ } => "currency",
        CurrencyInfo::OrigCurrency { code: _, rate: _ } => "origcurrency",
        &_ => "unknown currency type",
    };
    Value::record(
        record! {
            "code" => Value::string(c.code().as_str(), span),
            "rate" => convert_amount(&c.rate(), span),
            "type" => Value::string(orig, span)
        },
        span,
    )
}

// Primitives

fn convert_datetime(t: &OfxDateTime, span: Span) -> Value {
    Value::date(
        DateTime::from_timestamp_nanos(t.as_offset_date_time().unix_timestamp_nanos() as i64)
            .into(),
        span,
    )
}

fn convert_amount(a: &OfxAmount, span: Span) -> Value {
    Value::float(a.as_decimal().as_f64(), span)
}
