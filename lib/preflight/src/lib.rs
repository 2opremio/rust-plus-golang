extern crate libc;
extern crate soroban_env_host;

use std::convert::TryInto;
use soroban_env_host::budget::Budget;
use soroban_env_host::storage::{self, AccessType, SnapshotSource, Storage};
use soroban_env_host::xdr::{self, AccountId, HostFunction, LedgerEntry, LedgerKey, ReadXdr, ScHostStorageErrorCode, ScVec, WriteXdr};
use soroban_env_host::{Host, HostError, LedgerInfo};
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::rc::Rc;
use xdr::LedgerFootprint;

// TODO: we may want to pass callbacks instead of using global functions
extern "C" {
    // LedgerKey XDR in base64 string to LedgerEntry XDR in base64 string
    fn SnapshotSourceGet(ledger_key: *const libc::c_char) -> *const libc::c_char;
    // LedgerKey XDR in base64 string to bool
    fn SnapshotSourceHas(ledger_key: *const libc::c_char) -> libc::c_int;
    // Free Strings provided by Go
    fn FreeCString(str: *const libc::c_char);
}

struct CSnapshotSource;

impl SnapshotSource for CSnapshotSource {
    fn get(&self, key: &LedgerKey) -> Result<LedgerEntry, HostError> {
        let key_xdr = key.to_xdr_base64().unwrap();
        let key_cstr = CString::new(key_xdr).unwrap();
        let res = unsafe { SnapshotSourceGet(key_cstr.as_ptr()) };
        if res.is_null() {
            return Err(HostError::from(
                ScHostStorageErrorCode::AccessToUnknownEntry,
            ));
        }
        let res_cstr = unsafe { CStr::from_ptr(res) };
        let res_str = res_cstr.to_str().unwrap();
        // TODO: use a proper error
        let entry =
            LedgerEntry::from_xdr_base64(res_str).map_err(|_| ScHostStorageErrorCode::UnknownError)?;
        unsafe { FreeCString(res)};
        Ok(entry)
    }

    fn has(&self, key: &LedgerKey) -> Result<bool, HostError> {

        let key_xdr = key.to_xdr_base64().unwrap();
        let key_cstr = CString::new(key_xdr).unwrap();
        let res = unsafe { SnapshotSourceHas(key_cstr.as_ptr()) };
        Ok(match res {
            0 => false,
            _ => true,
        })
    }
}

#[repr(C)]
pub struct CLedgerInfo {
    pub protocol_version: u32,
    pub sequence_number: u32,
    pub timestamp: u64,
    pub network_passphrase: *const libc::c_char,
    pub base_reserve: u32,
}

impl From<CLedgerInfo> for LedgerInfo {
    fn from(c: CLedgerInfo) -> Self {
        let network_passphrase_cstr = unsafe { CStr::from_ptr(c.network_passphrase) };
        Self {
            protocol_version: c.protocol_version,
            sequence_number: c.sequence_number,
            timestamp: c.timestamp,
            network_passphrase: network_passphrase_cstr.to_str().unwrap().as_bytes().to_vec(),
            base_reserve: c.base_reserve,
        }
    }
}

fn storage_footprint_to_ledger_footprint(
    foot: &storage::Footprint,
) -> Result<LedgerFootprint, xdr::Error> {
    let mut read_only: Vec<LedgerKey> = Vec::new();
    let mut read_write: Vec<LedgerKey> = Vec::new();
    for (k, v) in foot.0.iter() {
        match v {
            AccessType::ReadOnly => read_only.push(*k.clone()),
            AccessType::ReadWrite => read_write.push(*k.clone()),
        }
    }
    Ok(LedgerFootprint {
        read_only: read_only.try_into()?,
        read_write: read_write.try_into()?,
    })
}


#[no_mangle]
pub extern "C" fn preflight_host_function(
    hf: *const libc::c_char,   // HostFunction XDR in base64
    args: *const libc::c_char, // ScVec XDR in base64
    source_account: *const libc::c_char, // AccountId XDR in base64
    ledger_info: CLedgerInfo,
) -> *mut libc::c_char // LedgerFootprint XDR in base64, TODO: use better error reporting
{
    let hf_cstr = unsafe { CStr::from_ptr(hf) };
    // TODO: remove _all_ the unwraps() around XDR decoding
    let hf = HostFunction::from_xdr_base64(hf_cstr.to_str().unwrap()).unwrap();
    let args_cstr = unsafe { CStr::from_ptr(args) };
    let args = ScVec::from_xdr_base64(args_cstr.to_str().unwrap()).unwrap();
    let source_account_cstr = unsafe { CStr::from_ptr(source_account) };
    let source_account = AccountId::from_xdr_base64(source_account_cstr.to_str().unwrap()).unwrap();
    let src = Rc::new(CSnapshotSource);
    let storage = Storage::with_recording_footprint(src);
    let budget = Budget::default();
    let host = Host::with_storage_and_budget(storage, budget);

    host.set_source_account(source_account);
    host.set_ledger_info(ledger_info.into());

    println!(
        "preflight execution of host function '{}'",
        HostFunction::name(&hf)
    );

    // Run the preflight.
    let res = host.invoke_function(hf, args);

    // Recover, convert and return the storage footprint and other values to C.
    let (storage, _, _) = match host.try_finish() {
        Ok(v) => v,
        Err(_) => {
            println!("finish failed");
            return null_mut();
        }
    };

    if let Err(err) = res {
        println!("preflight failed: {}", err);
        return null_mut();
    };

    let fp = match storage_footprint_to_ledger_footprint(&storage.footprint) {
        Ok(fp) => fp,
        Err(err) => {
            println!("footprint conversion failed: {}", err);
            return null_mut();
        }
    };
    let result =  CString::new(fp.to_xdr_base64().unwrap()).unwrap();
    // transfer ownership to caller
    // caller needs to invoke free_cstring(result) when done
    result.into_raw()
}

#[no_mangle]
pub extern "C" fn free_rust_cstring(str: *mut libc::c_char) {
    unsafe { let _ = CString::from_raw(str);}
}
