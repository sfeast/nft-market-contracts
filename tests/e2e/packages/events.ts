import { parseTokenMeta, sleep, getDeploy, getAccountInfo, getAccountNamedKeyValue } from "../utils";

import {
  CLValueBuilder,
  CLValueParsers,
  CLValue,
  CLTypeTag,
  CLMap
} from "casper-js-sdk";

export const EventParser = (
  {
    contractPackageHash,
    eventNames,
  }: { contractPackageHash: string; eventNames: any[] },
  value: any
) => {
  if (value.body.DeployProcessed.execution_result.Success) {
    const { transforms } =
      value.body.DeployProcessed.execution_result.Success.effect;

        const events = transforms.reduce((acc: any, val: any) => {
          if (
            val.transform.hasOwnProperty("WriteCLValue") &&
            typeof val.transform.WriteCLValue.parsed === "object" &&
            val.transform.WriteCLValue.parsed !== null
          ) {
            const maybeCLValue = CLValueParsers.fromJSON(
              val.transform.WriteCLValue
            );
            const clValue = maybeCLValue.unwrap();
            if (clValue && clValue.clType().tag === CLTypeTag.Map) {
              const hash = (clValue as CLMap<CLValue, CLValue>).get(
                CLValueBuilder.string("contract_package_hash")
              );
              const event = (clValue as CLMap<CLValue, CLValue>).get(CLValueBuilder.string("event_type"));
              if (
                hash &&
                // NOTE: Calling toLowerCase() because current JS-SDK doesn't support checksumed hashes and returns all lower case value
                // Remove it after updating SDK
                hash.value() === contractPackageHash.slice(5).toLowerCase() &&
                event &&
                eventNames.includes(event.value())
              ) {
                acc = [...acc, { name: event.value(), clValue }];
              }
            }
          }
          return acc;
        }, []);

    return { error: null, success: !!events.length, data: events };
  }

  return null;
};