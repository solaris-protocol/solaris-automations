use clap::{ 
    App, 
    Arg, 
    SubCommand,
    ArgMatches,
};

pub fn get_arg_matches<'a>() -> ArgMatches<'a> {
    App::new("client_rust")
        .about("Client for Solaris-Automation program on Solana")
        .version("1.0")
        .author("Max Fedyarov")
        .arg(Arg::with_name("config")
            .help("Sets a custom config file")
            .short("c")
            .long("config")
            .value_name("FILE")
            .takes_value(true))
        .arg(Arg::with_name("settings")
            .long("settings")
            .value_name("SETTINGS JSON")    
            .takes_value(true))
        .subcommand(SubCommand::with_name("fill_order")
            .arg(Arg::with_name("order")
                .value_name("ORDER JSON")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("create_order")
            .arg(Arg::with_name("order_base")
                .value_name("ORDER_BASE JSON")
                .takes_value(true)
                .required(true)))
        .get_matches()
}