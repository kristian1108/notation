# Design Philosophy

### 1 — Documentation should not be managed separately from the repo.
Documentation describes code. It should live with the code, have dinner with the code, go to bed
with the code, and get up in the morning with the code.

### 2 — I should not have to login to some separate service to update docs after I merge and deploy code. 
That's hard to keep track of. Engineers cannot be trusted to workout outside the repo. Online documentation tools
give me JIRA vibes. No thank youuu.

### 3 — The only frontend framework I want to think about is Markdown.
Docusaurus is an interesting product but it's too complicated. I do not want to wrestle with React.
I do not want to deploy a website.

### 4 — Some people are not engineers. They should be able to read my docs.
Users, product mommies, etc.

### 5 — Docs should be searchable. 
Good search is a hard problem so better let someone else worry about that.

### 6 — Documentation should be subject to the same review & CI process as code.
See point 2.

# Solution
Notation allows you to write markdown and automatically publish it Notion.

Once it's in Notion, it can be an internal thing, or you can ship it as a public website.

You also get all of Notion's AI, search, and formatting for free.

# Example
The website you're reading right now is managed by Notation.

# Usage
1. Write your documentation in markdown.
2. Create a Notion page to host your documentation.
3. Grab an API key from Notion (help below)
4. Throw that API key in a `Notation.toml` file (detail below)
5. Run `notation ship --src </path/to/you/docs>`


# FAQ
### What markdown features do you support?
- headers
- paragraphs
- code blocks
- lists (ordered, unordered)
- tables
- links
- images (although not local, you need to host yourself and put the link)
- relative page links (to other pages in the same repo, which will turn into Notion page links)
- arbitrary directory structure (will turn into subpages, subsubpages, etc.)

### How do I configure Notation?
Create a `Notation.toml` file in the root of your project.

```toml
# Notation.toml

[notion]
secret = ""
# this is the title of the page that will host your new documentation
parent_page = ""    
```

### How do I prep my Notion account to use Notation?
First, you need to have a notion account. Sign up here: [Notion](https://www.notion.so/)

Next, you need to create a page to host your documentation.

![](https://notation-media.s3.amazonaws.com/add_a_page.jpg)

Now, give that page a name:

![](https://notation-media.s3.amazonaws.com/notation_parent_name.jpg)

Finally, in your `Notation.toml` file, just write down this name (make sure it's a unique name within your space):

```toml
# Notation.toml

[notion]
secret = ""
parent_page = "Your Notation Parent" # <----- this name
```

### How do I get an API key from Notion?
You need to create an integration in Notion.

Go to `Settings & members` in the top right of your Notion home.

And then click `Connections` --> `Develop or manage integrations`.

![](https://notation-media.s3.amazonaws.com/add_integration.jpg)

Now you want to create a new integration.

![](https://notation-media.s3.amazonaws.com/new_integration.jpg)

Give it a name, assign it to one of your workspaces, and select `internal`
as the type of integration.

![](https://notation-media.s3.amazonaws.com/configure_integration.jpg)

Now just grab the secret:

![](https://notation-media.s3.amazonaws.com/grab_secret.jpg)

And throw it in your `Notation.toml`!

```toml
# Notation.toml

[notion]
secret = "your_new_integration_secret" # <----- right here
parent_page = "Your Notation Parent"
```