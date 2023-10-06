#!/usr/bin/env perl

# LaTeX
$latex = 'uplatex -interaction=nonstopmode -file-line-error %O %S';
$max_repeat = 5;

# # BibTeX
$bibtex = 'upbibtex %O %S';
$biber = 'biber --bblencoding=utf8 -u -U --output_safechars %O %S';

# DVI / PDF
# dvi -> pdf の変換に使用するものとしてdvipdfmxを指定
$dvipdf = 'dvipdfmx %O -o %D %S'; # dvipdfmx [options] -o output.pdf input.dvi ということになる
$pdf_mode = 3; # dviファイルからpdfを作成する場合 uplatexならこれ

# output directory 
# この.latexmkrcと同じディレクトリにあるoutディレクトリに出力する
use Cwd;
my $current_dir_path = Cwd::getcwd();
print "current directory path: $current_dir_path\n";

my $rc_file_path = "$current_dir_path/.latexmkrc"; # この.latexmkrcファイルの絶対パス
print "latexmkrc: $rc_file_path\n";
for (my $i = 0; $i < $#ARGV; $i++) {
    if ($ARGV[$i] eq '-r') {
        $rc_file_path = $ARGV[$i+1];
        $rc_file_path = ($rc_file_path =~m!^/!)?$rc_file_path : "$current_dir_path/$rc_file_path"; # 絶対パスに変換
        print "latexmkrc: $rc_file_path\n";
        last;
    }
}
$project_dir_path = $rc_file_path =~ s/\/\.latexmkrc$//r;

$out_dir = "$project_dir_path/out"; # e.g. /Users/username/project1/out
print "output directory path: $out_dir\n";


