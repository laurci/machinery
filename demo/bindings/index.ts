/*
AnalyzeResult {
    file_location: "crate",
    services: [
        Service {
            name: "hello",
            location: "crate::api::greeting",
            arguments: [
                "message: String",
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
    ],
}
*/

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

type Option<T> = T | undefined;
type Result<T> = T;
type Vec<T> = T[];

type String = string;
type Void = void;

type u8 = number;
type u16 = number;
type u32 = number;
type u64 = number;
type usize = number;

type i8 = number;
type i16 = number;
type i32 = number;
type i64 = number;
type isize = number;

type f32 = number;
type f64 = number;

type bool = boolean;
export type Thing = 
	"A" |
	"B" |
	"C";

export interface Greeting {
	message: String,
	thing: Option < Thing >,
}

export function createClient(transport: Transport) {
	let obj0 = {};
	let obj1 = Object.assign(obj0, { api: { greeting: { async hello(message: String): Promise<Result < Greeting >> { return handleResult(await transport.send("api::greeting::hello", JSON.stringify([message ?? null]))); } }}});
	let obj2 = Object.assign(obj1, { api: { greeting: { async hi(): Promise<Result < Void >> { return handleResult(await transport.send("api::greeting::hi", JSON.stringify([]))); } }}});
	return obj2;
};
