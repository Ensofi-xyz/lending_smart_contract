use super::duration_to_year;

pub struct TotalRepayLoanAmountParams {
    pub borrower_fee_percent: f64,
    pub interest: f64,
    pub duration: u64,
    pub borrow_amount: u64,
}

pub fn get_total_repay_loan_amount(params: TotalRepayLoanAmountParams) -> u64 {
    let TotalRepayLoanAmountParams {
        borrower_fee_percent,
        interest,
        duration,
        borrow_amount,
    } = params;

    let borrower_fee_percent = borrower_fee_percent / 100.0;

    let loan_interest_percent = interest / 100.0;

    let time_borrowed = duration_to_year(duration);

    let interest_amount = (borrow_amount as f64) * loan_interest_percent * time_borrowed;

    let borrower_fee_amount = borrower_fee_percent * interest_amount;

    return (borrow_amount as f64 + interest_amount + borrower_fee_amount) as u64;
}
