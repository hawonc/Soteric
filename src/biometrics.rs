use anyhow::{anyhow, Result};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Bool};
use objc2::msg_send;
use objc2_foundation::NSString;
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};
use std::sync::mpsc;

#[link(name = "LocalAuthentication", kind = "framework")]
unsafe extern "C" {}

const SERVICE_NAME: &str = "com.soteric.encryption-key";
const ACCOUNT_NAME: &str = "soteric-biometric-key";
const LA_POLICY_BIOMETRICS: isize = 1;

fn authenticate_touch_id() -> Result<()> {
    let cls = AnyClass::get(c"LAContext")
        .ok_or_else(|| anyhow!("LAContext not available — is LocalAuthentication linked?"))?;

    let context: Retained<AnyObject> = unsafe { msg_send![cls, new] };

    let mut error: *mut AnyObject = std::ptr::null_mut();
    let can: Bool = unsafe {
        msg_send![&context, canEvaluatePolicy: LA_POLICY_BIOMETRICS, error: &mut error]
    };

    if !can.as_bool() {
        return Err(anyhow!("Touch ID is not available on this device"));
    }

    let reason = NSString::from_str("Soteric: unlock encryption key");

    let (tx, rx) = mpsc::channel();

    let block = RcBlock::new(move |success: Bool, _error: *mut AnyObject| {
        let _ = tx.send(success.as_bool());
    });

    unsafe {
        let _: () = msg_send![
            &context,
            evaluatePolicy: LA_POLICY_BIOMETRICS,
            localizedReason: &*reason,
            reply: &*block
        ];
    }

    let success = rx
        .recv()
        .map_err(|_| anyhow!("Touch ID response channel closed unexpectedly"))?;

    if success {
        Ok(())
    } else {
        Err(anyhow!("Touch ID authentication failed or was cancelled"))
    }
}

pub fn store_biometric_secret(secret: &str) -> Result<()> {
    authenticate_touch_id()?;

    let _ = delete_generic_password(SERVICE_NAME, ACCOUNT_NAME);

    set_generic_password(SERVICE_NAME, ACCOUNT_NAME, secret.as_bytes())
        .map_err(|e| anyhow!("Failed to store secret in Keychain: {}", e))?;

    Ok(())
}

pub fn retrieve_biometric_secret() -> Result<String> {
    let data = get_generic_password(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|_| anyhow!("No biometric secret found. Run 'setup-biometric' first."))?;

    authenticate_touch_id()?;

    String::from_utf8(data).map_err(|_| anyhow!("Keychain secret is not valid UTF-8"))
}

pub fn delete_biometric_secret() -> Result<()> {
    delete_generic_password(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| anyhow!("Failed to delete secret from Keychain: {}", e))?;
    Ok(())
}
