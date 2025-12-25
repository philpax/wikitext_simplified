export interface Example {
  name: string;
  wikitext: string;
}

export const examples: Example[] = [
  {
    name: 'Basic Formatting',
    wikitext: `This is '''bold''', this is ''italic'', and this is '''''bold italic'''''.

Here's some <sup>superscript</sup> and <sub>subscript</sub> text.`,
  },
  {
    name: 'Links',
    wikitext: `Check out the [[Main Page]] or [[Wikipedia|the free encyclopedia]].

External links work too: [https://example.com Example Site]`,
  },
  {
    name: 'Headings',
    wikitext: `== Section ==
Some content here.

=== Subsection ===
More content.

==== Sub-subsection ====
Even more content.`,
  },
  {
    name: 'Templates',
    wikitext: `{{Infobox|name=Example|type=Demo}}

Using a template parameter: {{{description|No description provided}}}`,
  },
  {
    name: 'Lists',
    wikitext: `Unordered list:
* Item 1
* Item 2
** Nested item
* Item 3

Ordered list:
# First
# Second
# Third

Definition list:
;Term 1
:Definition 1
;Term 2
:Definition 2`,
  },
  {
    name: 'Table',
    wikitext: `{| class="wikitable"
|+ Caption
|-
! Header 1 !! Header 2
|-
| Cell 1 || Cell 2
|-
| Cell 3 || Cell 4
|}`,
  },
  {
    name: 'Complex Example',
    wikitext: `== Overview ==

'''Michigan rap''' is a subgenre of [[Midwestern hip-hop]] in the United States.

=== Notable Artists ===
* [[J Dilla]]
* [[Eminem]]
* [[Big Sean]]

{| class="wikitable"
|+ Key Albums
|-
! Artist !! Album !! Year
|-
| J Dilla || Donuts || 2006
|-
| Eminem || The Marshall Mathers LP || 2000
|}

In 2023, ''[[Rolling Stone]]'' described Michigan rap as "the regional style of intense punchlines."`,
  },
  {
    name: 'Code Block',
    wikitext: `<syntaxhighlight lang="lua">
function hello()
    print("Hello, world!")
end
</syntaxhighlight>`,
  },
];
