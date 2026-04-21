use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};

pub fn run<T: HttpTransport>(client: &YtClient<T>, opts: &OutputOptions) -> Result<(), YtdError> {
    let user = client.get_me()?;
    format::print_single(&user, opts);
    Ok(())
}
