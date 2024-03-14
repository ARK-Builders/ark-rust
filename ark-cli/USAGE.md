# Usage

## Get started

Create an empty dir:
```
mkdir /tmp/test
cd /tmp/test
```

Let's fill it with something. One of the simplest ways to create resources it is to save a link to web page using `ark-cli link` command:
```
$ ark-cli link create . http://google.com goo
$ ark-cli link create . http://duckduckgo.com duck
```

We can use `ark-cli list` to see just created resources:
```
22-207093268
18-1909444406
```

These are just ids, derived from the URLs themselves.

Now, the dir structure should resemble this:
```
/tmp/test
└───.ark
    ├───cache
    │   ├───metadata
    │   └───previews
    │
    └───user
        ├───properties
        ├───scores
        └───tags
```

### Label your data 

You can attach various metadata to your data, e.g. tags:
```
$ ark-cli file append . tags 22-207093268 search,engine
```

The same way we can append scores:
```
$ ark-cli file append . scores 22-207093268 15
```

Generic metadata is possible using JSON-based properties:
```
$ ark-cli file append . properties 22-207093268 favorites:false,ai:true --format=json
```

### Navigate your data

The simplest command to observe your resources is `list`:
```
$ ark-cli list

18-1909444406
22-207093268
```

You can also target this command to other folders:
```
$ ark-cli list ~/Pictures/

58922-3276384608
62591-2492670715
723145-720506115
125308-3041567246
```

But it's a bit boring and doesn't really tell anything, right? Various flags should be used to gain more knowledge about your collections of resources:
* `--entry=id|path|both|link` to show the path,the id or both of a resource
* `--timestamp=true` to show or not the last modified timestamp of a resource
* `--tags=true` to show or not the tags for every resource
* `--scores=true` to show or not the scores for every resource
* `--sort=asc|desc` to sort resources by asc or dsc order of scores
* `--filter=query` to filter resources by their tags

For instance, you can list files with their paths and attached tags:
```
$ ark-cli list -pt

30-4257856154 search
18-1909444406 hello
22-207093268  search,engine
38-103010298  NO_TAGS
```

You Can list the links of the files

```
$ark-cli list -l

https://google.com
https://news.ycombinator.com
https://youtube.com
https://github.com

```

Or, sort by score:
```
$ ark-cli list -s --sort=asc

30-4257856154 NO_SCORE
18-1909444406 2
38-103010298  10
22-207093268  15
```

Finally, you can filter resources using their tags:
```
$ /tmp/ark-cli list -t --filter=search

30-4257856154 search
22-207093268  search,engine
```

## :zap: Low-level utilities :zap:

There are commands which could be useful with time, when you grasp the basic concepts. Some of these commands also can be useful for debugging [ArkLib](https://github.com/ARK-Builders/ark-rust).

### Retrieve the metadata

You can read these properties:
```
$ ark-cli file read . properties 22-207093268
{"ai":"true","desc":null,"favorites":"false","title":"duck"}
```

As well as scores or tags:
```
$ ark-cli file read . scores 22-207093268
15
$ ark-cli file read . tags 22-207093268
search,engine
```

### Inspect storages

It's also possible to list resources having some metadata in a particular storage:
```
$ ark-cli storage list . properties
22-207093268
18-1909444406

$ ark-cli storage list . tags
22-207093268

$ ark-cli storage list . scores
22-207093268
```

Note that, in this example, resource with id `18-1909444406` is listed only in `properties` storage since it lacks any metadata in `tags` and `scores` storages. The `ark-cli storage list` command only lists entries of a particular storage, not all resources.

### Inspect versions

For delving into history of storage mutations, we made `--versions` flag:
```
$ ark-cli storage list . properties --versions=true
version  name           machine                              path
2        22-207093268   0592a937-a5d1-4843-8f03-ae0d6a9e77b5 ./.ark/user/properties/22-207093268/22-207093268_0592a937-a5d1-4843-8f03-ae0d6a9e77b5.2
1        18-1909444406  0592a937-a5d1-4843-8f03-ae0d6a9e77b5 ./.ark/user/properties/18-1909444406/18-1909444406_0592a937-a5d1-4843-8f03-ae0d6a9e77b5.1
```

Each storage mutation made by `ark-cli file append` or `ark-cli file insert` commands increases the number in `version` column. Versions help to prevent dirty-writes caused by using same storages by separate apps, or devices. 

The `properties` storage is _folder-based_, but same command can be used with _file-based_ storages like `tags`:
```
$ ark-cli storage list . tags --versions=true
Loading app id at /home/kirill/.ark...
id               value
22-207093268     search,engine

$ ark-cli file append . tags 22-207093268 wow
$ ark-cli storage list . tags --versions=true
id               value
22-207093268     search,engine
22-207093268     wow

$ ark-cli file append . tags 22-207093268 one_more_time
$ ark-cli storage list . tags --versions=true
id               value
22-207093268     search,engine
22-207093268     wow
22-207093268     one_more_time
```
