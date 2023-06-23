/*
AnalyzeResult {
    file_location: "crate",
    services: [
        Service {
            name: "hello",
            location: "crate::api::greeting",
            arguments: [
                "message: String",
                "input: GreetingInput",
            ],
            return_type: "Result < Greeting >",
        },
        Service {
            name: "hi",
            location: "crate::api::greeting",
            arguments: [],
            return_type: "Result < Void >",
        },
    ],
    messages: [
        Message {
            kind: Enum,
            name: "Thing",
            location: "crate::api::greeting",
            code: "export type Thing = \n\t\"A\" |\n\t\"B\" |\n\t\"C\";\n",
        },
        Message {
            kind: Struct,
            name: "Greeting",
            location: "crate::api::greeting",
            code: "export interface Greeting {\n\tmessage: String,\n\tthing: Option < Thing >,\n}\n",
        },
        Message {
            kind: Struct,
            name: "GreetingInput",
            location: "crate::api::greeting",
            code: "export interface GreetingInput {\n\tname: String,\n}\n",
        },
    ],
}
*/
/* custom header */
export interface Transport {
	send(fn: string, args: string): Promise<string>;
}

function handleResult(result: string) {
	const json = JSON.parse(result);
	if (json.error) {
		throw new Error(json.error);
	}
	return json.result;
}

export type Option<T> = T | undefined;
export type Result<T> = T;
export type Vec<T> = T[];

export type String = string;
export type Void = void;

export type u8 = number;
export type u16 = number;
export type u32 = number;
export type u64 = number;
export type usize = number;

export type i8 = number;
export type i16 = number;
export type i32 = number;
export type i64 = number;
export type isize = number;

export type f32 = number;
export type f64 = number;

export type bool = boolean;
export type Custom = string;
export type Thing = 
	"A" |
	"B" |
	"C";

export interface Greeting {
	message: String,
	thing: Option < Thing >,
}

export interface GreetingInput {
	name: String,
}

export function createClient(transport: Transport) {
	let obj0 = {};
	let obj1 = Object.assign(obj0, { api: { greeting: { async hello(message: String, input: GreetingInput): Promise<Result < Greeting >> { return handleResult(await transport.send("api::greeting::hello", JSON.stringify([message ?? null, input ?? null]))); } }}});
	let obj2 = Object.assign(obj1, { api: { greeting: { async hi(): Promise<Result < Void >> { return handleResult(await transport.send("api::greeting::hi", JSON.stringify([]))); } }}});
	return obj2;
};
/* custom_footer */