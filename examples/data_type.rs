//#[macro_use]
extern crate redismodule;

use redismodule::{Context, Command, RedisResult, NextArg};
use redismodule::native_types::RedisType;
use redismodule::redismodule::RedisValue;

#[derive(Debug)]
struct MyType {
    data: String,
}

static MY_REDIS_TYPE: RedisType = RedisType::new("mytype123");

fn alloc_set(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = args.next_string()?;
    let size = args.next_i64()?;

    ctx.log_debug(format!("key: {}, size: {}", key, size).as_str());

    let key = ctx.open_key_writable(&key);

    match key.get_value::<MyType>(&MY_REDIS_TYPE)? {
        None => {
            let value = MyType {
                data: "A".repeat(size as usize)
            };

            key.set_value(&MY_REDIS_TYPE, value)?;
        }
        Some(value) => {
            value.data = "B".repeat(size as usize);
        }
    }

    Ok(size.into())
}

fn alloc_get(ctx: &Context, args: Vec<String>) -> RedisResult {
    let mut args = args.into_iter().skip(1);
    let key = args.next_string()?;

    let key = ctx.open_key_writable(&key); // TODO: Use read-only key

    match key.get_value::<MyType>(&MY_REDIS_TYPE)? {
        None => Ok(RedisValue::None),
        Some(value) => {
            // TODO: Use the value
            let _ = value;
            Ok("some value".into())
        }
    }
}

//////////////////////////////////////////////////////

const MODULE_NAME: &str = "alloc";
const MODULE_VERSION: c_int = 1;

/*
redis_module!(
    MODULE_NAME,
    MODULE_VERSION,
    [
        &MY_REDIS_TYPE,
    ],
    [
        Command::new("alloc.set", alloc_set, "write"),
    ]
);
*/

use std::os::raw::c_int;
use std::ffi::CString;
use std::slice;

use redismodule::raw;
use redismodule::RedisString;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn RedisModule_OnLoad(
    ctx: *mut raw::RedisModuleCtx,
    _argv: *mut *mut raw::RedisModuleString,
    _argc: c_int,
) -> c_int {
    unsafe {
        ///////////////////////////////////////
        let module_name = MODULE_NAME;
        let module_version = MODULE_VERSION;
        let data_types = vec![&MY_REDIS_TYPE];

        ///////////////////////////////////////

        let module_name = CString::new(module_name).unwrap();
        let module_version = module_version as c_int;

        if raw::Export_RedisModule_Init(
            ctx,
            module_name.as_ptr(),
            module_version,
            raw::REDISMODULE_APIVER_1 as c_int,
        ) == raw::Status::Err as _ { return raw::Status::Err as _; }

        for data_type in data_types {
            if data_type.create_data_type(ctx).is_err() {
                return raw::Status::Err as _;
            }
        }

        if true {
            redismodule::alloc::use_redis_alloc();
        } else {
            eprintln!("*** NOT USING Redis allocator ***");
        }

        //////////////////////////////
        // Command 1
        {
            ///////////////////////////
            let command_name = "alloc.set";
            let command_flags = "write";
            ///////////////////////////

            let name = CString::new(command_name).unwrap();
            let flags = CString::new(command_flags).unwrap();
            let (firstkey, lastkey, keystep) = (1, 1, 1);

            /////////////////////
            extern fn do_command(
                ctx: *mut raw::RedisModuleCtx,
                argv: *mut *mut raw::RedisModuleString,
                argc: c_int,
            ) -> c_int {
                let context = Context::new(ctx);

                let args: Vec<String> = unsafe { slice::from_raw_parts(argv, argc as usize) }
                    .into_iter()
                    .map(|a| RedisString::from_ptr(*a).expect("UTF8 encoding error in handler args").to_string())
                    .collect();

                let response = alloc_set(&context, args);
                context.reply(response) as c_int
            }
            /////////////////////

            if raw::RedisModule_CreateCommand.unwrap()(
                ctx,
                name.as_ptr(),
                Some(do_command),
                flags.as_ptr(),
                firstkey, lastkey, keystep,
            ) == raw::Status::Err as _ { return raw::Status::Err as _; }
        }
        //////////////////////////////

        //////////////////////////////
        // Command 2
        {
            ///////////////////////////
            let command_name = "alloc.get";
            let command_flags = "write";
            ///////////////////////////

            let name = CString::new(command_name).unwrap();
            let flags = CString::new(command_flags).unwrap();
            let (firstkey, lastkey, keystep) = (1, 1, 1);

            /////////////////////
            extern fn do_command(
                ctx: *mut raw::RedisModuleCtx,
                argv: *mut *mut raw::RedisModuleString,
                argc: c_int,
            ) -> c_int {
                let context = Context::new(ctx);

                let args: Vec<String> = unsafe { slice::from_raw_parts(argv, argc as usize) }
                    .into_iter()
                    .map(|a| RedisString::from_ptr(*a).expect("UTF8 encoding error in handler args").to_string())
                    .collect();

                let response = alloc_get(&context, args);
                context.reply(response) as c_int
            }
            /////////////////////

            if raw::RedisModule_CreateCommand.unwrap()(
                ctx,
                name.as_ptr(),
                Some(do_command),
                flags.as_ptr(),
                firstkey, lastkey, keystep,
            ) == raw::Status::Err as _ { return raw::Status::Err as _; }
        }
        //////////////////////////////

        raw::Status::Ok as _
    }
}