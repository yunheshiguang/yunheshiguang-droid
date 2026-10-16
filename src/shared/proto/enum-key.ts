import type { DescEnum } from "@bufbuild/protobuf";

type EnumObject = { UNSPECIFIED: number };

type EnumKey<E> = Extract<keyof E, string>;

export function enumKey<E extends EnumObject>(values: E, value: number): EnumKey<E> {
  const name = (values as unknown as Record<number, string | undefined>)[value];

  return (name ?? "UNSPECIFIED") as EnumKey<E>;
}

export function enumKeys<E extends EnumObject>(values: E): Exclude<EnumKey<E>, "UNSPECIFIED">[] {
  return Object.keys(values).filter(
    (key) => Number.isNaN(Number(key)) && key !== "UNSPECIFIED",
  ) as Exclude<EnumKey<E>, "UNSPECIFIED">[];
}

export function enumKeyOfName(schema: DescEnum, protoName: string): string {
  return schema.values.find((value) => value.name === protoName)?.localName ?? "UNSPECIFIED";
}
