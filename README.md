# Large Wet Dog

A discord bot that fixes some embed fails.

## Usage

Uses the `LWD_TOKEN` environment variable to login.

Remember to give your instance of LWD permissions to read messages in the discord developer portal.

Listens for a few types of embed fails:

- Links to x.com with no embed, a blocked embed, or an embed with the thumbnail to a video/gallery get replied to with a vxtwitter.com link to the same tweet.

  - This has the minor issue that the art trend of splitting wide images into a carousel of thin images gets displayed properly by the x.com embed but not by the vxtwitter.com embed.

- Links to tumblr.com with no embed or a blocked embed get replied to with a txtumblr.com link to the same post.

- Links to vxtwitter.com that show the first page of a tweet's images in a message with no vxtwitter.com link to the second page get replied to with a vxtwitter.com link to the second page.

When a message is edited (such as to upstream the embed unfail) LWD updates/deletes its reply accordingly.

When a message is deleted, LWD deletes its reply.

When a message contains `||`, LWD assumes the entire message is spoilered and spoilers its entire reply.

## Why the name?

I needed a name and my brain came up with "Large Wet Dog". It means nothing.
