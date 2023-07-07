/*
AnalyzeResult {
    file_location: "crate",
    services: [
        Service {
            name: "format",
            location: "crate::api::greeting",
            arguments: [
                "message: String",
                "input: GreetingInput",
            ],
            return_type: "Result < Greeting >",
        },
        Service {
            name: "say_hi",
            location: "crate::api::greeting",
            arguments: [],
            return_type: "Result < Void >",
        },
    ],
    messages: [
        Message {
            kind: Enum,
            name: "TimeOfDay",
            location: "crate::api::greeting",
            code: "export type TimeOfDay = \n\t\"Morning\" |\n\t\"Afternoon\" |\n\t\"Evening\";\n",
        },
        Message {
            kind: Struct,
            name: "Greeting",
            location: "crate::api::greeting",
            code: "export interface Greeting {\n\tmessage: String,\n\ttime_of_day: Option < TimeOfDay >,\n}\n",
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

const deepAssign: typeof Object.assign = (target: any, ...sources: any[]) => {
    for (const source of sources) {
        for (let k in source) {
            let vs = source[k], vt = target[k];
            if (Object(vs) == vs && Object(vt) === vt) {
                target[k] = deepAssign(vt, vs);
                continue;
            }
            target[k] = source[k];
        }
    }
    return target;
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
export type TimeOfDay = 
	"Morning" |
	"Afternoon" |
	"Evening";

export interface Greeting {
	message: String,
	time_of_day: Option < TimeOfDay >,
}

export interface GreetingInput {
	name: String,
}

export function createClient(transport: Transport) {
	let obj0 = {};
	let obj1 = deepAssign(obj0, { greeting: { async format(message: String, input: GreetingInput): Promise<Result < Greeting >> { return handleResult(await transport.send("api::greeting::format", JSON.stringify([message ?? null, input ?? null]))); } }});
	let obj2 = deepAssign(obj1, { greeting: { async say_hi(): Promise<Result < Void >> { return handleResult(await transport.send("api::greeting::say_hi", JSON.stringify([]))); } }});
	return obj2;
};
/* custom_footer */