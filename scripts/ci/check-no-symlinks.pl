#!/usr/bin/env perl
use strict;
use warnings;
use Cwd qw(abs_path);
use Errno qw(ENOENT);
use Getopt::Long qw(GetOptions);

my $consumer;
Getopt::Long::Configure(qw(no_auto_abbrev no_ignore_case));
GetOptions('consumer=s' => \$consumer) && !@ARGV && defined($consumer)
    or die "usage: check-no-symlinks.pl --consumer ROOT\n";
$consumer !~ /[\r\n]/ or die "consumer directory contains CR/LF\n";
my $root = abs_path($consumer);
defined($root) && -d $root && $root !~ /[\r\n]/ or die "invalid consumer directory\n";
chdir $root or die "cannot enter consumer: $!\n";
delete @ENV{qw(GIT_DIR GIT_WORK_TREE GIT_COMMON_DIR GIT_INDEX_FILE GIT_OBJECT_DIRECTORY
    GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_NAMESPACE)};
$ENV{GIT_OPTIONAL_LOCKS} = '0';

sub git_read {
    open my $fh, '-|', 'git', @_ or die "cannot start Git: $!\n";
    local $/;
    my $bytes = <$fh> // '';
    close $fh or die "Git inventory failed\n";
    return $bytes;
}

git_read('rev-parse', '--show-toplevel') eq "$root\n"
    or die "consumer must be a Git checkout root\n";
my $index = git_read('ls-files', '--stage', '-z');
my $working = git_read('ls-files', '--cached', '--others', '--exclude-standard', '-z');
my (%paths, %violations);
for my $record (split /\0/, $index) {
    $record =~ /\A([0-7]{6}) [0-9a-f]+ ([0-3])\t(.+)\z/s or die "invalid Git index record\n";
    my ($mode, $stage, $path) = ($1, $2, $3);
    $stage == 0 or die "unmerged index path: $path\n";
    $violations{$path} = 1 if $mode eq '120000';
    $paths{$path} = 1;
}
$paths{$_} = 1 for split /\0/, $working;
for my $path (sort keys %paths) {
    $path !~ m{\A/|(?:\A|/)\.{1,2}(?:/|\z)|//|/\z} or die "invalid Git inventory path\n";
    my $prefix = '';
    for my $part (split m{/}, $path) {
        $prefix = length($prefix) ? "$prefix/$part" : $part;
        unless (lstat $prefix) {
            # Tracked deletions are allowed; inaccessible observations are not.
            last if $! == ENOENT;
            die "cannot inspect $prefix: $!\n";
        }
        if (-l _) { $violations{$prefix} = 1; last; }
    }
}
die "Repository symlinks are forbidden:\n" . join('', map { "$_\n" } sort keys %violations)
    if %violations;
print "No symlinks in the index or tracked/nonignored working paths.\n";
