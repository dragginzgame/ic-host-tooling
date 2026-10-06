#!/usr/bin/env perl
use strict;
use warnings;
use FindBin;
use File::Temp qw(tempdir);
use File::Path qw(remove_tree);
use Test::More;

my $checker = "$FindBin::Bin/check-documentation-links.pl";
my $fixture = tempdir('documentation-links.XXXXXX', TMPDIR => 1, CLEANUP => 0);
END {
    if (defined $fixture) {
        if (Test::More->builder->is_passing) { remove_tree($fixture); }
        else { warn "Documentation link fixtures retained: $fixture\n"; }
    }
}
mkdir "$fixture/docs" or die $!;
sub write_file {
    my ($path, $text) = @_;
    open my $file, '>', "$fixture/$path" or die $!;
    print {$file} $text;
    close $file or die $!;
}
sub check {
    return system($^X, $checker, '--root', $fixture, @_);
}
write_file('target file.md', 'target');
write_file('docs/links.md', <<"MARKDOWN");
[relative](../target%20file.md#anchor "title")
![image](<../target file.md> 'title')
[reference][ref]
[ref]: <../target file.md> "title"
[absolute](<$fixture/target file.md>)
[scheme](https://example.invalid/no-file) [anchor](#missing)
[protocol-relative](//example.invalid/path)
`[inline code](missing.md)`
   ~~~markdown
[example](missing.md)
   ~~~~
````markdown
[example](missing.md)
```
[still fenced](missing.md)
````
MARKDOWN
is(check('docs/links.md'), 0, 'relative, absolute, encoded, titled and reference links; fences and code ignored');
is(check("$fixture/docs/links.md"), 0, 'absolute document input');
write_file('docs/links.md', "Completely different prose. [link](../target%20file.md?raw=1)\n");
is(check('docs/links.md'), 0, 'prose and query strings do not change local existence contract');
write_file('docs/links.md', "[missing](absent.md)\n");
isnt(check('docs/links.md'), 0, 'missing inline target fails');
write_file('docs/links.md', "[ref]: <absent file.md> 'title'\n");
isnt(check('docs/links.md'), 0, 'missing reference target fails');
isnt(check('missing-document.md'), 0, 'missing document fails');
isnt(check('docs'), 0, 'directory cannot masquerade as an empty document');
write_file('docs/links.md', "~~~\n[unterminated code](absent.md)\n");
is(check('docs/links.md'), 0, 'unclosed code fence extends to EOF');
isnt(check(), 0, 'no implicit document roster');
done_testing();
