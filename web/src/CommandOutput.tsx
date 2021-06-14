import React from "react";
import AutoScroll from "@brianmcallister/react-auto-scroll";
import { Anchor, Text, Main } from "grommet";

// regex from https://github.com/sindresorhus/linkify-urls
const urlRegex =
  /((?<!\+)(?:https?(?::\/\/))(?:www\.)?(?:[a-zA-Z\d-_.]+(?:(?:\.|@)[a-zA-Z\d]{2,})|localhost)(?:(?:[-a-zA-Z\d:%_+.~#*$!?&//=@]*)(?:[,](?![\s]))*)*)/g;

function linkify(text: string) {
  return text
    .split(urlRegex)
    .map((x, index) =>
      index % 2 === 1 ? <Anchor target="_blank" href={x} label={x} /> : x
    );
}

export const CommandOutput = ({ lines }: { lines: string[] }) => {
  return (
    <Main>
      <AutoScroll height={800} showOption={false}>
        {lines.map((msg) => {
          return (
            <Text key={msg} as="pre" size="small" margin="xsmall">
              {linkify(msg)}
            </Text>
          );
        })}
      </AutoScroll>
    </Main>
  );
};
